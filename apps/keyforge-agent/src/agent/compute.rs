// apps/keyforge-agent/src/agent/compute.rs

use keyforge_infra::AssetManager;
use anyhow::Result;
use keyforge_protocol::JobConfig;
use keyforge_runner::{OptimizationRunner, RunnerOptions};
use keyforge_model::OptimizationResult;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::info;
use crate::models::{SharedTelemetry, ComputeConfig};

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
    async fn sync_assets(&self, config: &JobConfig, limits: &ComputeConfig) -> Result<(String, String)> {
        if config.corpora.len() > limits.max_corpora_sources {
            return Err(anyhow::anyhow!("Too many corpora (limit {})", limits.max_corpora_sources));
        }
        if config.corpora.is_empty() {
            return Err(anyhow::anyhow!("job config has no corpora specified"));
        }
        self.sync_job_assets(config)
            .await
            .map_err(|e| anyhow::anyhow!(e))
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
    session: keyforge_core::ScoringSession,
    job_id: String,
    stop_flag: Arc<AtomicBool>,
    limiter: Arc<Semaphore>,
    telemetry: SharedTelemetry,
    timeout_sec: u64,
    log_sampling_rate: usize,
    config: &JobConfig,
) -> Result<OptimizationResult> {
    // Acquire permit to respect core limits
    let _permit = limiter.acquire().await.map_err(|_| anyhow::anyhow!("Semaphore closed"))?;

    info!(job_id = %job_id, "starting optimization loop via runner");

    let logger = crate::agent::telemetry::WorkerLogger {
        stop_flag: stop_flag.clone(),
        job_id: job_id.clone(),
        telemetry: telemetry.clone(),
        sample_rate: log_sampling_rate,
    };

    let options = RunnerOptions {
        timeout_sec,
        log_sampling_rate,
        ..Default::default()
    };

    OptimizationRunner::run(session, job_id, stop_flag, logger, options, config)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}
