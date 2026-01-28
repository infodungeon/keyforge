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

use keyforge_compute::{AssetLoader, SessionBuilder};
use keyforge_infra::asset::ValkeyProvider;
use keyforge_model::{
    constants::{
        DEFAULT_CORPUS_WEIGHT, VERIFICATION_TOLERANCE_ABS_MIN, VERIFICATION_TOLERANCE_RATIO,
    },
    CorpusSource, KeyboardDefinition, KeycodeRegistry,
};
use keyforge_physics::ScoringEngine;
use keyforge_protocol::ResultSubmission;
use keyforge_security as crypto;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::warn;

use crate::cache::{CompiledEngineCache, ParsedLayoutCache};
use crate::error::{AppError, AppResult};
use crate::infra::repositories::{JobRepository, NodeRepository};

#[derive(Clone, Debug)]
pub struct VerificationService {
    jobs: JobRepository,
    nodes: NodeRepository,
    assets: Arc<ValkeyProvider>,
    engine_cache: Arc<CompiledEngineCache>,
    layout_cache: Arc<ParsedLayoutCache>,
    compilation_semaphore: Arc<Semaphore>,
}

impl VerificationService {
    #[must_use]
    pub fn new(
        jobs: JobRepository,
        nodes: NodeRepository,
        assets: Arc<ValkeyProvider>,
        engine_cache: Arc<CompiledEngineCache>,
        layout_cache: Arc<ParsedLayoutCache>,
        max_concurrent_compilations: usize,
    ) -> Self {
        Self {
            jobs,
            nodes,
            assets,
            engine_cache,
            layout_cache,
            compilation_semaphore: Arc::new(Semaphore::new(max_concurrent_compilations)),
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
        // 1. Replay Protection (Task-hive-rev-003: Persistent tracking)
        // check_and_set_nonce returns true if it's NEW, false if it's a REPLAY
        let is_new = self
            .assets
            .get_coordinator()
            .check_and_set_nonce(
                &sub.node_id,
                sub.nonce,
                i64::try_from(crate::config::DEFAULT_SUBMISSION_EXPIRATION_SECS).unwrap_or(300),
            )
            .await
            .map_err(|e| AppError::Internal(format!("Coordination error: {e}")))?;

        if !is_new {
            return Err(AppError::Validation(
                "Nonce already used (Replay attack detected)".into(),
            ));
        }

        let public_key = self
            .nodes
            .get_public_key(&sub.node_id)
            .await
            .map_err(AppError::Database)?;
        let pk = public_key.ok_or_else(|| {
            AppError::Validation("Unregistered Node Identity: Public Key Required".into())
        })?;

        let valid = crypto::verify_result_fixed(
            &pk,
            &sub.job_id,
            &sub.layout,
            sub.raw_score,
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

        // --- ATOMIC COMPILATION BOUNDARY ---
        // We limit concurrent compilations to prevent CPU exhaustion (DOS protection)
        let _permit = self
            .compilation_semaphore
            .acquire()
            .await
            .map_err(|e| AppError::Internal(format!("Semaphore error: {e}")))?;

        // Re-check cache after acquiring permit (Double-Checked Locking pattern)
        if let Some(engine) = self.engine_cache.get(&sub.job_id) {
            return self.check_tolerance(engine.clone(), sub).await;
        }

        let (geometry, weights, corpus_name, cost_source) = self
            .jobs
            .get_config(&sub.job_id)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::NotFound)?;

        let builder = SessionBuilder::new(self.assets.as_ref())
            .with_keyboard_def(Arc::new(KeyboardDefinition {
                name: String::default(),
                author: String::default(),
                version: String::default(),
                notes: String::default(),
                kb_type: String::default(),
                geometry,
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

        let layout_struct = if let Some(cached) = self.layout_cache.get(&sub.layout) {
            cached
        } else {
            let registry = self
                .assets
                .load::<KeycodeRegistry>(keycodes_file)
                .await
                .map_err(|e| {
                    AppError::Validation(format!("Failed to load keycodes for verification: {e}"))
                })?;

            let parsed = keyforge_adapter::conversion::parse_layout_string_strict(
                &sub.layout,
                engine.key_count(),
                &registry,
            )
            .map_err(|e| AppError::Validation(format!("Layout parse error: {e}")))?;

            self.layout_cache.insert(&sub.layout, parsed.clone());
            parsed
        };

        let calculated_score = engine
            .score(&layout_struct)
            .map_err(|e| AppError::Validation(format!("Scoring error: {e}")))?;

        let is_exact = engine.capabilities().is_exact;

        if is_exact {
            if calculated_score.0 != sub.raw_score {
                warn!(
                    "❌ Bit-perfect Mismatch: Claimed {} vs Calc {}",
                    sub.raw_score, calculated_score.0
                );
                return Err(AppError::Validation(
                    "Bit-perfect score verification failed".into(),
                ));
            }
        } else {
            let calculated_f32 = calculated_score.to_f32();
            let diff = (calculated_f32 - sub.score).abs();
            let tolerance =
                (sub.score * VERIFICATION_TOLERANCE_RATIO).max(VERIFICATION_TOLERANCE_ABS_MIN);

            if diff > tolerance {
                warn!(
                    "❌ Score Mismatch (approx): Claimed {:.4} vs Calc {:.4} (Diff: {:.4})",
                    sub.score, calculated_f32, diff
                );
                return Err(AppError::Validation(
                    "Approximate score verification failed".into(),
                ));
            }
        }
        Ok(())
    }
}
