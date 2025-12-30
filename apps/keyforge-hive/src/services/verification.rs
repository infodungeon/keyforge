use crate::cache::{CompiledEngineCache, GlobalAssetCache};
use crate::error::{AppError, AppResult};
use crate::infra::repositories::{JobRepository, NodeRepository};
use keyforge_protocol::{
    config::CorpusSource,
    constants::{VERIFICATION_TOLERANCE_ABS_MIN, VERIFICATION_TOLERANCE_RATIO},
    CostMatrixSource, ResultSubmission,
};
use keyforge_compute::SessionBuilder;
use keyforge_infra::AssetLoader;
use keyforge_security as crypto;
use keyforge_core::ScoringEngine; // Corrected import
use std::sync::Arc;
use tracing::warn;

/// Service responsible for verifying the correctness and authenticity of node submissions.
#[derive(Clone)]
pub struct VerificationService {
    jobs: JobRepository,
    nodes: NodeRepository,
    assets: Arc<GlobalAssetCache>,
    engine_cache: Arc<CompiledEngineCache>,
}

impl VerificationService {
    pub fn new(
        jobs: JobRepository,
        nodes: NodeRepository,
        assets: Arc<GlobalAssetCache>,
        engine_cache: Arc<CompiledEngineCache>,
    ) -> Self {
        Self {
            jobs,
            nodes,
            assets,
            engine_cache,
        }
    }

    /// Verifies both the signature and the calculated score of a result submission.
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

        if let Some(sig) = &sub.signature {
            let valid = crypto::verify_result(
                &pk,
                &sub.job_id,
                &sub.layout,
                sub.score,
                sub.timestamp,
                sub.nonce,
                sig,
            )
            .map_err(|e| AppError::Validation(format!("Crypto Error: {}", e)))?;

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

        let (geometry, weights, corpus_name, cost_raw) = self
            .jobs
            .get_config(&sub.job_id)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::NotFound)?;

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
        let kb_def = keyforge_protocol::geometry::KeyboardDefinition {
            meta: Default::default(),
            geometry,
            layouts: Default::default(),
        };

        let session = builder.build_preloaded(
            &kb_def,
            &[CorpusSource { id: corpus_name, weight: 1.0, hash: None }],
            &weights,
            &keyforge_protocol::config::SearchParams::default(),
            "keycodes.json",
            &cost_source,
            None
        ).await.map_err(|e| AppError::Validation(format!("Session build failed: {}", e)))?;

        self.engine_cache.insert(&sub.job_id, session.engine.clone());
        self.check_tolerance(session.engine, sub).await
    }

    async fn check_tolerance(
        &self,
        engine: Arc<ScoringEngine>,
        sub: &ResultSubmission,
    ) -> AppResult<()> {
        let registry = self.assets.load_keycodes("keycodes.json").await
            .unwrap_or_else(|_| keyforge_model::keycodes::KeycodeRegistry::new_with_defaults());

        let layout_struct = keyforge_adapter::conversion::parse_layout_string_strict(
            &sub.layout,
            engine.key_count(),
            &registry,
        )
        .map_err(|e| AppError::Validation(format!("Layout parse error: {}", e)))?;

        let calculated_score = engine.score(&layout_struct)
            .map_err(|e| AppError::Validation(format!("Scoring error: {}", e)))?;

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
