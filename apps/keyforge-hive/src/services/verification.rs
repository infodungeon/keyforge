// apps/keyforge-hive/src/services/verification.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


use crate::cache::CompiledEngineCache;
use crate::error::{AppError, AppResult};
use crate::infra::repositories::{JobRepository, NodeRepository};
use keyforge_model::{
    CorpusSource,
    constants::{VERIFICATION_TOLERANCE_ABS_MIN, VERIFICATION_TOLERANCE_RATIO},
    CostMatrixSource, KeyboardDefinition, SearchParams
};
use keyforge_protocol::ResultSubmission;
use keyforge_compute::SessionBuilder;
use keyforge_infra::{AssetLoader, ValkeyProvider};
use keyforge_security as crypto;
use keyforge_core::ScoringEngine;
use std::sync::Arc;
use tracing::warn;

/// A service responsible for validating submitted optimization results.
///
/// It performs cryptographic signature verification and re-scores the submitted 
/// layout to ensure the claimed score is within an acceptable tolerance.
#[derive(Clone)]
pub struct VerificationService {
    jobs: JobRepository,
    nodes: NodeRepository,
    assets: Arc<ValkeyProvider>, // CHANGED
    engine_cache: Arc<CompiledEngineCache>,
}

impl VerificationService {
    /// Creates a new `VerificationService` with the required repositories and caches.
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

    /// Performs a full verification of a result submission.
    ///
    /// This includes checking the Ed25519 signature and re-calculating the 
    /// score using the same physics engine configuration used for the job.
    pub async fn verify_submission(&self, sub: &ResultSubmission) -> AppResult<()> {
        self.verify_signature(sub).await?;
        self.verify_score(sub).await?;
        Ok(())
    }

    async fn verify_signature(&self, sub: &ResultSubmission) -> AppResult<()> {
        let public_key = self.nodes.get_public_key(&sub.node_id).await.map_err(AppError::Database)?;
        let pk = public_key.ok_or_else(|| {
            AppError::Validation("Unregistered Node Identity: Public Key Required".into())
        })?;

        if let Some(sig) = &sub.signature {
            let valid = crypto::verify_result(
                &pk,
                &sub.job_id,
                &sub.layout,
                sub.score,
                sub.timestamp,
                sub.nonce,
                sig,
            ).map_err(|e| AppError::Validation(format!("Crypto Error: {}", e)))?;

            if !valid {
                return Err(AppError::Validation("Invalid Signature".into()));
            }
        } else {
            return Err(AppError::Validation("Missing Signature".into()));
        }
        Ok(())
    }

    async fn verify_score(&self, sub: &ResultSubmission) -> AppResult<()> {
        if let Some(engine) = self.engine_cache.get(&sub.job_id) {
            return self.check_tolerance(engine.clone(), sub).await;
        }

        let (geometry, weights, corpus_name, cost_raw) = self.jobs.get_config(&sub.job_id).await.map_err(AppError::Database)?.ok_or(AppError::NotFound)?;

        let cost_source = if cost_raw.trim().starts_with('[') || cost_raw.trim().starts_with('{') {
            if let Ok(src) = serde_json::from_str::<CostMatrixSource>(&cost_raw) {
                src
            } else {
                CostMatrixSource::Predefined(cost_raw)
            }
        } else {
            CostMatrixSource::Predefined(cost_raw)
        };

        let builder = SessionBuilder::new(self.assets.as_ref());
        let kb_def = KeyboardDefinition {
            meta: Default::default(),
            geometry,
            layouts: Default::default(),
        };

        let session = builder.build_preloaded(
            &kb_def,
            &[CorpusSource { id: corpus_name, weight: 1.0, hash: None }],
            &weights,
            &SearchParams::default(),
            "keycodes.json",
            &cost_source,
            None
        ).await.map_err(|e| AppError::Validation(format!("Session build failed: {}", e)))?;

        self.engine_cache.insert(&sub.job_id, session.engine.clone());
        self.check_tolerance(session.engine, sub).await
    }

    async fn check_tolerance(&self, engine: Arc<ScoringEngine>, sub: &ResultSubmission) -> AppResult<()> {
        let registry = self.assets.load_keycodes("keycodes.json").await
            .unwrap_or_else(|_| keyforge_model::keycodes::KeycodeRegistry::new_with_defaults());

        let layout_struct = keyforge_adapter::conversion::parse_layout_string_strict(
            &sub.layout,
            engine.key_count(),
            &registry,
        ).map_err(|e| AppError::Validation(format!("Layout parse error: {}", e)))?;

        let calculated_score = engine.score(&layout_struct).map_err(|e| AppError::Validation(format!("Scoring error: {}", e)))?;

        let diff = (calculated_score - sub.score).abs();
        let tolerance = (sub.score * VERIFICATION_TOLERANCE_RATIO).max(VERIFICATION_TOLERANCE_ABS_MIN);

        if diff > tolerance {
            warn!("❌ Score Mismatch: Claimed {:.4} vs Calc {:.4} (Diff: {:.4})", sub.score, calculated_score, diff);
            return Err(AppError::Validation("Score verification failed".into()));
        }
        Ok(())
    }
}
