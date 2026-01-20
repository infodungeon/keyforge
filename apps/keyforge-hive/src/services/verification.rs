// apps/keyforge-hive/src/services/verification.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::cache::CompiledEngineCache;
use crate::error::{AppError, AppResult};
use crate::infra::repositories::{JobRepository, NodeRepository};
use keyforge_compute::SessionBuilder;
use keyforge_core::ScoringEngine;
use keyforge_infra::{AssetLoader, ValkeyProvider};
use keyforge_model::{
    constants::{
        DEFAULT_CORPUS_WEIGHT, VERIFICATION_TOLERANCE_ABS_MIN, VERIFICATION_TOLERANCE_RATIO,
    },
    CorpusSource, CostMatrixSource, KeyboardDefinition, KeyboardMeta, KeycodeRegistry,
};
use keyforge_protocol::ResultSubmission;
use keyforge_security as crypto;
use std::sync::Arc;
use tracing::warn;

#[derive(Clone, Debug)]
pub struct VerificationService {
    jobs: JobRepository,
    nodes: NodeRepository,
    assets: Arc<ValkeyProvider>,
    engine_cache: Arc<CompiledEngineCache>,
}

impl VerificationService {
    #[must_use]
    pub fn new(
        jobs: JobRepository,
        nodes: NodeRepository,
        assets: Arc<ValkeyProvider>,
        engine_cache: Arc<CompiledEngineCache>,
    ) -> Self {
        Self {
            jobs,
            nodes,
            assets,
            engine_cache,
        }
    }

    /// Verifies a result submission by checking its signature and score.
    ///
    /// # Errors
    ///
    /// Returns `AppResult` if signature verification or score tolerance check fails.
    pub async fn verify_submission(&self, sub: &ResultSubmission) -> AppResult<()> {
        self.verify_signature(sub).await?;
        self.verify_score(sub).await?;
        Ok(())
    }

    async fn verify_signature(&self, sub: &ResultSubmission) -> AppResult<()> {
        let public_key = self
            .nodes
            .get_public_key(&sub.node_id)
            .await
            .map_err(AppError::Database)?;
        let pk = public_key.ok_or_else(|| {
            AppError::Validation("Unregistered Node Identity: Public Key Required".into())
        })?;

        let valid = crypto::verify_result(
            &pk,
            &sub.job_id,
            &sub.layout,
            sub.score,
            sub.timestamp,
            sub.nonce,
            &sub.signature,
        )
        .map_err(|e| AppError::Validation(format!("Crypto Error: {e}")))?;

        if !valid {
            return Err(AppError::Validation("Invalid Signature".into()));
        }
        Ok(())
    }

    async fn verify_score(&self, sub: &ResultSubmission) -> AppResult<()> {
        if let Some(engine) = self.engine_cache.get(&sub.job_id) {
            return self.check_tolerance(engine.clone(), sub).await;
        }

        let (geometry, weights, corpus_name, cost_raw) = self
            .jobs
            .get_config(&sub.job_id)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::NotFound)?;

        let cost_source = serde_json::from_str::<CostMatrixSource>(&cost_raw)
            .unwrap_or(CostMatrixSource::Predefined(cost_raw));

        let builder = SessionBuilder::new(self.assets.as_ref())
            .with_keyboard_def(Arc::new(KeyboardDefinition {
                meta: KeyboardMeta::default(),
                geometry,
                layouts: std::collections::HashMap::default(),
            }))
            .with_corpus(&[CorpusSource {
                id: corpus_name,
                weight: DEFAULT_CORPUS_WEIGHT,
                hash: None,
            }])
            .await
            .map_err(|e| AppError::Validation(format!("Corpus load failed: {e}")))?
            .with_cost_matrix(&cost_source)
            .await
            .map_err(|e| AppError::Validation(format!("Cost matrix load failed: {e}")))?
            .with_keycodes(keyforge_model::constants::ASSET_KEYCODES_FILENAME)
            .await
            .map_err(|e| AppError::Validation(format!("Keycodes load failed: {e}")))?
            .with_rubric(keyforge_adapter::conversion::to_domain_rubric(&weights))
            .with_config(keyforge_model::SearchConfig::default());

        let session = builder
            .build()
            .map_err(|e| AppError::Validation(format!("Session build failed: {e}")))?;

        self.engine_cache
            .insert(&sub.job_id, session.engine.clone());
        self.check_tolerance(session.engine, sub).await
    }

    async fn check_tolerance(
        &self,
        engine: Arc<dyn ScoringEngine>,
        sub: &ResultSubmission,
    ) -> AppResult<()> {
        let keycodes_file = keyforge_model::constants::ASSET_KEYCODES_FILENAME;

        let registry = self
            .assets
            .load::<KeycodeRegistry>(keycodes_file)
            .await
            .map_err(|e| {
                AppError::Validation(format!("Failed to load keycodes for verification: {e}"))
            })?;

        let layout_struct = keyforge_adapter::conversion::parse_layout_string_strict(
            &sub.layout,
            engine.key_count(),
            &registry,
        )
        .map_err(|e| AppError::Validation(format!("Layout parse error: {e}")))?;

        let calculated_score = engine
            .score(&layout_struct)
            .map_err(|e| AppError::Validation(format!("Scoring error: {e}")))?
            .to_f32();

        let diff = (calculated_score - sub.score).abs();
        let tolerance =
            (sub.score * VERIFICATION_TOLERANCE_RATIO).max(VERIFICATION_TOLERANCE_ABS_MIN);

        if diff > tolerance {
            warn!(
                "❌ Score Mismatch: Claimed {:.4} vs Calc {:.4} (Diff: {:.4})",
                sub.score, calculated_score, diff
            );
            return Err(AppError::Validation("Score verification failed".into()));
        }
        Ok(())
    }
}
