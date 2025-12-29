use crate::cache::{CompiledEngineCache, GlobalAssetCache};
use crate::error::{AppError, AppResult};
use crate::infra::repositories::{JobRepository, NodeRepository};
use keyforge_protocol::{
    config::CorpusSource,
    constants::{VERIFICATION_TOLERANCE_ABS_MIN, VERIFICATION_TOLERANCE_RATIO},
    CostMatrixSource, ResultSubmission,
};
use keyforge_security as crypto;
use std::sync::Arc;
use tracing::warn;

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
            return self.check_tolerance(engine.clone(), sub);
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

        let domain_kb = keyforge_adapter::conversion::to_domain_keyboard(&geometry);

        use keyforge_model::loader::AssetLoader;
        let corpus = self
            .assets
            .load_corpus(&[CorpusSource {
                id: corpus_name.clone(),
                weight: 1.0,
                hash: None,
            }])
            .map_err(|e| AppError::Validation(format!("Corpus load failed: {}", e)))?;

        let raw_cost_data = match &cost_source {
            CostMatrixSource::Predefined(name) => self
                .assets
                .load_cost_matrix(name)
                .map_err(|e| AppError::Validation(format!("Cost matrix load failed: {}", e)))?,
            CostMatrixSource::Custom(_) => keyforge_model::loader::RawCostData { entries: vec![] },
        };

        let overrides =
            keyforge_adapter::conversion::resolve_cost_matrix(&raw_cost_data.entries, &geometry);

        let domain_rubric = keyforge_adapter::conversion::to_domain_rubric(&weights);

        let engine =
            keyforge_core::ScoringEngine::new(&domain_kb, &corpus, &domain_rubric, &overrides)
            .map_err(|e| AppError::Validation(format!("Physics error: {}", e)))?;
        let engine_arc = Arc::new(engine);

        self.engine_cache.insert(&sub.job_id, engine_arc.clone());

        self.check_tolerance(engine_arc, sub)
    }

    fn check_tolerance(
        &self,
        engine: Arc<keyforge_core::ScoringEngine>,
        sub: &ResultSubmission,
    ) -> AppResult<()> {
        use keyforge_model::loader::AssetLoader;
        let registry = self
            .assets
            .load_keycodes("keycodes.json")
            .unwrap_or_else(|_| keyforge_protocol::keycodes::KeycodeRegistry::new_with_defaults());

        let layout_struct = keyforge_adapter::conversion::parse_layout_string_strict(
            &sub.layout,
            engine.key_count(),
            &registry,
        )
        .map_err(|e| AppError::Validation(format!("Layout parse error: {}", e)))?;

        let calculated_score = engine.score(&layout_struct);

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
