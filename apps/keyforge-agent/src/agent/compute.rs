// apps/keyforge-agent/src/agent/compute.rs

use crate::models::{ComputeConfig, SharedTelemetry};
use anyhow::Result;
use keyforge_compute::{Runtime, ScoringSession};
use keyforge_infra::AssetManager;
use keyforge_model::types::SpatialUnit;
use keyforge_model::OptimizationResult;
use keyforge_protocol::{CostMatrixSourceDto, JobConfig};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::info;

/// A trait for types that can synchronize assets required for an optimization job.
pub trait AssetSyncer {
    /// Syncs assets for the given job configuration, returning the cost matrix filename
    /// and the corpora bundle directory name.
    fn sync_assets(
        &self,
        config: &JobConfig,
        limits: &ComputeConfig,
    ) -> impl std::future::Future<Output = Result<(String, String)>> + Send;
}

impl AssetSyncer for AssetManager {
    async fn sync_assets(
        &self,
        config: &JobConfig,
        limits: &ComputeConfig,
    ) -> Result<(String, String)> {
        if config.corpora.len() > limits.max_corpora_sources {
            return Err(anyhow::anyhow!(
                "Too many corpora (limit {})",
                limits.max_corpora_sources
            ));
        }
        if config.corpora.is_empty() {
            return Err(anyhow::anyhow!("job config has no corpora specified"));
        }
        self.sync_job_assets(config)
            .await
            .map_err(|e: keyforge_infra::error::InfraError| anyhow::anyhow!(e))?;

        // Extract cost matrix name and primary corpus
        let cost_name = match &config.cost_matrix {
            CostMatrixSourceDto::Predefined(s) => s.clone(),
        };
        let corpus_id = config.corpora[0].id.clone();

        Ok((cost_name, corpus_id))
    }
}

/// Helper to prepare all assets required for a job.
pub async fn prepare_assets<S: AssetSyncer>(
    syncer: &S,
    config: &JobConfig,
    limits: &ComputeConfig,
) -> Result<(String, String)> {
    syncer.sync_assets(config, limits).await
}

/// Executes the core optimization loop for a job using the centralized Runner.
pub async fn run_optimization(
    session: ScoringSession,
    job_id: String,
    stop_flag: Arc<AtomicBool>,
    limiter: Arc<Semaphore>,
    telemetry: SharedTelemetry,
    _timeout_sec: u64,
    log_sampling_rate: usize,
    config: &JobConfig,
) -> Result<OptimizationResult> {
    // Acquire permit to respect core limits with timeout
    let permit_res =
        tokio::time::timeout(std::time::Duration::from_secs(5), limiter.acquire()).await;

    let _permit = match permit_res {
        Ok(Ok(p)) => p,
        Ok(Err(_)) => return Err(anyhow::anyhow!("Semaphore closed")),
        Err(_) => return Err(anyhow::anyhow!("Timeout waiting for concurrency permit")),
    };

    info!(job_id = %job_id, "starting optimization loop via runner");

    let logger = crate::agent::telemetry::WorkerLogger {
        stop_flag: stop_flag.clone(),
        job_id: job_id.clone(),
        telemetry: telemetry.clone(),
        sample_rate: log_sampling_rate,
    };

    // Use consolidated compute runner
    let runtime = Runtime::from(session);
    runtime
        .run_optimization(logger, &config.to_domain_pinned_keys())
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;
    use crate::models::AgentTelemetry;
    use keyforge_model::cost_model::CostModel;
    use keyforge_model::{KeyIndex, KeyNode, Keyboard, KeycodeRegistry};
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;
    use tokio::sync::Semaphore;

    #[tokio::test]
    async fn test_compute_optimization_run() {
        let kb_def = keyforge_model::KeyboardDefinition {
            geometry: keyforge_model::KeyboardGeometry {
                keys: vec![KeyNode {
                    index: 0,
                    x: SpatialUnit::from_f32(0.0),
                    y: SpatialUnit::from_f32(0.0),
                    ..Default::default()
                }],
                prime_slots: vec![KeyIndex::new(0)],
                med_slots: vec![],
                low_slots: vec![],
                home_row: keyforge_model::types::RowIndex::new(0),
            },
            ..Default::default()
        };

        let kb = Arc::new(
            Keyboard::new(
                kb_def.geometry.keys.clone(),
                kb_def.geometry.home_row,
                "test".into(),
            )
            .unwrap(),
        );

        let cost_json = r#"{
            "meta": { "version": "2.0", "description": "Test", "unit": "pts" },
            "models": {
                "model_a_row_staggered": {
                    "description": "Test Model",
                    "static_costs": {
                        "universal_hand": {
                            "thumb": { "pos_1": 100.0 },
                            "index": { "base": { "r0": 100.0 } },
                            "middle": { "base": { "r0": 100.0 } },
                            "ring": { "base": { "r0": 100.0 } },
                            "pinky": { "base": { "r0": 100.0 } }
                        }
                    }
                }
            },
            "dynamic_rules": { "sequence_modifiers": {}, "penalties": {}, "constraints": {} }
        }"#;
        let cost_model: Arc<CostModel> = Arc::new(serde_json::from_str(cost_json).unwrap());

        let engine: Arc<dyn keyforge_physics::ScoringEngine> =
            keyforge_physics::EngineFactory::new_generic(
                &keyforge_physics::EngineCompilationContext {
                    keyboard: kb,
                    corpus: Arc::new(keyforge_model::Corpus::default()),
                    rubric: Arc::new(keyforge_model::Rubric::default()),
                    cost_model,
                    engine_config: keyforge_model::config::EngineConfig::default(),
                },
            )
            .unwrap()
            .into();

        let search_config = keyforge_model::SearchConfig::Annealing {
            steps: 10,
            start_temp: 10.0,
            end_temp: 0.1,
            seed: 42,
            patience: 10,
            reheats: 0,
            reheat_factor: 0.5,
            include_thumbs: false,
        };

        let session =
            ScoringSession::new(engine, Arc::new(KeycodeRegistry::default()), search_config);

        let job_config = JobConfig {
            definition: kb_def.into(),
            weights: keyforge_model::config::ScoringWeights::default().into(),
            params: keyforge_model::config::SearchParams::default().into(),
            pinned_keys: vec![].into(),
            corpora: vec![].into(),
            cost_matrix: CostMatrixSourceDto::Predefined("test".to_string()),
            biometrics: vec![].into(),
            parent_job_id: None,
            baseline_score: None,
            parents: vec![].into(),
        };

        let stop_flag = Arc::new(AtomicBool::new(false));
        let limiter = Arc::new(Semaphore::new(1));
        let telemetry = Arc::new(AgentTelemetry::default());

        let result = run_optimization(
            session,
            "job-123".to_string(),
            stop_flag,
            limiter,
            telemetry,
            3600,
            100,
            &job_config,
        )
        .await
        .expect("Optimization should complete successfully");

        assert!(result.score >= 0.0);
    }
}