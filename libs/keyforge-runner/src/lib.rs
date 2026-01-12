// libs/keyforge-runner/src/lib.rs

use anyhow::Result;
use keyforge_core::{ProgressCallback, ScoringSession};
use keyforge_core::loader::AssetLoader;
use keyforge_model::OptimizationResult;
use keyforge_protocol::JobConfig;
use keyforge_compute::SessionBuilder;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{info, error, warn};

/// Configuration for running an optimization task.
#[derive(Debug, Clone)]
pub struct RunnerOptions {
    /// Maximum time allowed for optimization.
    pub timeout_sec: u64,
    /// Number of CPU cores to utilize (0 for auto).
    pub threads: usize,
    /// Random seed for deterministic runs (optional).
    pub seed: Option<u64>,
    /// How often to emit progress updates (e.g. every N steps).
    pub log_sampling_rate: usize,
    /// Filename for the keycode registry asset.
    pub keycodes_file: String,
}

impl Default for RunnerOptions {
    fn default() -> Self {
        Self {
            timeout_sec: 3600,
            threads: 0,
            seed: None,
            log_sampling_rate: 1000,
            keycodes_file: "default".to_string(),
        }
    }
}

/// A unified service for preparing and executing KeyForge optimization jobs.
#[derive(Debug)]
pub struct OptimizationRunner;

impl OptimizationRunner {
    /// Hydrates a protocol-level JobConfig into a ready-to-run ScoringSession.
    ///
    /// This performs validation and asset loading using the provided loader.
    pub async fn prepare_session(
        loader: &dyn AssetLoader,
        config: &JobConfig,
        options: &RunnerOptions,
    ) -> Result<ScoringSession> {
        let builder = SessionBuilder::new(loader);
        
        let session = builder.build_preloaded(
            &config.definition,
            &config.corpora,
            &config.weights,
            &config.params,
            &options.keycodes_file,
            &config.cost_matrix,
            options.seed
        ).await.map_err(|e| anyhow::anyhow!(e))?;

        Ok(session)
    }

    /// Spawns a dedicated thread to run the optimization and handles the lifecycle.
    pub async fn run<CB>(
        session: ScoringSession,
        job_id: String,
        stop_flag: Arc<AtomicBool>,
        callback: CB,
        options: RunnerOptions,
        config: &JobConfig,
    ) -> Result<OptimizationResult> 
    where 
        CB: ProgressCallback + Send + 'static
    {
        info!(job_id = %job_id, "starting runner task");

        let timeout = tokio::time::Duration::from_secs(options.timeout_sec);
        let (tx, rx) = tokio::sync::oneshot::channel();

        let job_id_inner = job_id.clone();
        
        // Resolve Initial Layout
        let initial_layout = if !config.parents.is_empty() {
            use rand::prelude::IndexedRandom;
            let mut rng = rand::rng();
            config.parents.choose(&mut rng)
                .map(|s| keyforge_adapter::conversion::parse_layout_string_strict(s, session.engine.key_count(), &session.registry))
                .transpose()?
        } else {
            config.definition.layouts.get("default")
                .map(|s| keyforge_adapter::conversion::parse_layout_string_strict(s, session.engine.key_count(), &session.registry))
                .transpose()?
        };

        // Resolve Pinned Keys
        let pinned_keys = keyforge_adapter::conversion::resolve_constraints(&config.pinned_keys, session.engine.key_count(), &session.registry)
            .map_err(|e| anyhow::anyhow!(e))?;

        // Validate Constraints against initial layout
        if let Some(layout) = &initial_layout {
            for pin in pinned_keys.iter().flatten() {
                if !layout.keys.contains(pin) {
                     return Err(anyhow::anyhow!("Pinned key '{}' not found in initial layout", session.registry.get_label(*pin)));
                }
            }
        } else if pinned_keys.iter().any(|p| p.is_some()) {
             return Err(anyhow::anyhow!("Pinned keys provided but no initial layout found."));
        }

        // Spawn a real OS thread for CPU-bound work
        std::thread::Builder::new()
            .name(format!("kf-opt-{}", job_id))
            .spawn(move || {
                let result = catch_unwind(AssertUnwindSafe(|| {
                    keyforge_core::optimize_with_engine(
                        session.engine, 
                        &session.search_config, 
                        callback,
                        initial_layout,
                        Some(&pinned_keys)
                    )
                }));
                
                let final_res = match result {
                    Ok(res) => res.map_err(|e| anyhow::anyhow!(e.to_string())),
                    Err(payload) => {
                        let msg = if let Some(s) = payload.downcast_ref::<&str>() {
                            format!("panic: {}", s)
                        } else if let Some(s) = payload.downcast_ref::<String>() {
                            format!("panic: {}", s)
                        } else {
                            "unknown panic type".to_string()
                        };
                        error!(job_id = %job_id_inner, error = %msg, "runner thread panicked");
                        Err(anyhow::anyhow!(msg))
                    }
                };
                let _ = tx.send(final_res);
            })?;

        // Wait for result or timeout
        match tokio::time::timeout(timeout, rx).await {
            Ok(channel_res) => {
                channel_res.map_err(|_| anyhow::anyhow!("runner thread disconnected unexpectedly"))?
            },
            Err(_) => {
                warn!(job_id = %job_id, "runner timed out, signalling stop");
                stop_flag.store(true, Ordering::SeqCst);
                Err(anyhow::anyhow!("optimization timed out after {} seconds", options.timeout_sec))
            }
        }
    }
}
