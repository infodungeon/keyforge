// apps/keyforge-agent/src/agent/compute.rs

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

use keyforge_infra::AssetManager;
use anyhow::{Context, Result};
use keyforge_adapter::conversion;
use keyforge_core::EngineRequest;
use keyforge_persistence::UserRepo;
use keyforge_core::loader::AssetLoader;
use keyforge_model::OptimizationResult;
use keyforge_protocol::JobConfig;
use keyforge_model::Validator;
use std::future::Future;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{info, error}; // Added error
use rand::seq::SliceRandom;
use crate::models::{SharedTelemetry, ComputeConfig};

/// A trait for types that can synchronize assets required for an optimization job.
pub trait AssetSyncer {
    /// Syncs assets for the given job configuration, returning the cost matrix filename
    /// and the corpora bundle directory name.
    fn sync_assets(
        &self,
        config: &JobConfig,
        limits: &ComputeConfig,
    ) -> impl Future<Output = Result<(String, String)>> + Send;
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
            .context("asset sync failed")
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

/// Represents a job that has been hydrated with all necessary domain models
/// and is ready to be passed to the optimization engine.
#[derive(Debug)]
pub struct PreparedJob {
    /// The hydrated engine request.
    pub req: EngineRequest,
    /// The keycode registry used for token mapping.
    pub registry: Arc<keyforge_model::keycodes::KeycodeRegistry>,
    /// The active keyboard model.
    pub keyboard: Arc<keyforge_model::Keyboard>,
    /// The active corpus model.
    pub corpus: Arc<keyforge_model::Corpus>,
    /// The active scoring rubric.
    pub rubric: Arc<keyforge_model::Rubric>,
    /// Resolved cost model overrides.
    pub cost_overrides: Vec<(usize, usize, f32)>,
}

/// Loads assets from the filesystem, validates them, and constructs an `EngineRequest`.
pub async fn create_engine_request(
    loader: Box<dyn AssetLoader>,
    data_root: PathBuf,
    config: &JobConfig,
    cost_filename: &str,
    _corpus_root: &str,
    compute_config: &ComputeConfig,
) -> Result<PreparedJob> {
    config.weights.validate().map_err(|e| anyhow::anyhow!("invalid weights: {}", e))?;
    config.params.validate().map_err(|e| anyhow::anyhow!("invalid params: {}", e))?;
    config.definition.geometry.validate().map_err(|e| anyhow::anyhow!("invalid geometry: {}", e))?;

    // [Fixed] Removed unnecessary clone. UserRepo takes ownership of PathBuf.
    let user_data = UserRepo::new(data_root);
    let kb_name = &config.definition.meta.name;
    let safe_kb_name = keyforge_infra::sanitize_filename(kb_name);

    let model_def: keyforge_model::geometry::KeyboardDefinition = 
        serde_json::from_value(serde_json::to_value(&config.definition)?)?;

    user_data.save_keyboard_definition(&safe_kb_name, &model_def).context("failed to save keyboard definition")?;

    let start_layout = if !config.parents.is_empty() {
        let mut rng = rand::thread_rng();
        config.parents.choose(&mut rng).cloned()
    } else {
        None
    };

    // Note: loader needs data_root if it wasn't pre-configured, but here loader is passed in.
    // If loader needed data_root, it should have been configured with it.
    // data_root is consumed by UserRepo above.
    
    let definition = loader.load_keyboard(&safe_kb_name).await.context("failed to load keyboard definition")?;

    let domain_corpora: Vec<keyforge_model::config::CorpusSource> = config.corpora
        .iter()
        .map(conversion::to_domain_corpus_source)
        .collect();

    let corpus = loader.load_corpus(&domain_corpora).await.context("failed to load corpus")?;
    let raw_cost = loader.load_cost_matrix(cost_filename).await.context("failed to load cost matrix")?;
    let registry = loader.load_keycodes(&compute_config.keycodes_file).await.unwrap_or_else(|_| Arc::new(keyforge_model::keycodes::KeycodeRegistry::new_with_defaults()));

    let keyboard = keyforge_model::Keyboard::new(definition.geometry.keys.clone(), definition.geometry.home_row)
        .map_err(|e| anyhow::anyhow!("Invalid keyboard definition: {}", e))?;
    
    let rubric = conversion::to_domain_rubric(&config.weights);
    let cost_overrides = raw_cost.resolve(&definition.geometry);
    let pinned_keys = conversion::resolve_constraints(&config.pinned_keys, keyboard.keys.len(), &registry).map_err(|e| anyhow::anyhow!(e))?;
    
    // Use default_search_seed from config
    let search_config = conversion::to_domain_config(&config.params, compute_config.default_search_seed);

    let initial_layout = match start_layout {
        Some(layout_str) => Some(conversion::parse_layout_string(&layout_str, keyboard.keys.len(), &registry).map_err(|e| anyhow::anyhow!(e))?),
        None => definition.layouts.get("default").map(|default_str| conversion::parse_layout_string(default_str, keyboard.keys.len(), &registry).map_err(|e| anyhow::anyhow!(e))).transpose()?,
    };

    if let Some(layout) = &initial_layout {
        for pin in pinned_keys.iter().flatten() {
            if !layout.keys.contains(pin) {
                 return Err(anyhow::anyhow!("Pinned key '{}' not found in initial layout", registry.get_label(*pin)));
            }
        }
    } else if pinned_keys.iter().any(|p| p.is_some()) {
         return Err(anyhow::anyhow!("Pinned keys provided but no initial layout found."));
    }

    let keyboard = Arc::new(keyboard);
    let corpus = corpus;
    let rubric = Arc::new(rubric);

    let req = EngineRequest {
        keyboard: keyboard.clone(),
        corpus: corpus.clone(),
        rubric: rubric.clone(),
        config: search_config,
        initial_layout,
        pinned_keys,
        cost_overrides: cost_overrides.clone(),
    };

    Ok(PreparedJob { req, registry, keyboard, corpus, rubric, cost_overrides })
}

/// Executes the core optimization loop for a job.
pub async fn run_optimization(
    req: EngineRequest,
    job_id: String,
    stop_flag: Arc<AtomicBool>,
    limiter: Arc<Semaphore>,
    telemetry: SharedTelemetry,
    timeout_sec: u64,
    log_sampling_rate: usize,
) -> Result<OptimizationResult> {
    // Acquire permit to respect core limits
    let _permit = limiter.acquire().await.map_err(|_| anyhow::anyhow!("Semaphore closed"))?;

    info!(job_id = %job_id, "starting optimization loop");

    let logger = crate::agent::telemetry::WorkerLogger {
        stop_flag: stop_flag.clone(),
        job_id: job_id.clone(),
        telemetry: telemetry.clone(),
        sample_rate: log_sampling_rate,
    };

    let timeout = tokio::time::Duration::from_secs(timeout_sec);
    let job_id_inner = job_id.clone();

    // [Fixed] Use std::thread::spawn instead of spawn_blocking.
    // This isolates the optimization thread from the Tokio blocking pool, preventing
    // pool starvation if the physics engine hangs.
    let (tx, rx) = tokio::sync::oneshot::channel();

    std::thread::Builder::new()
        .name(format!("opt-{}", job_id))
        .spawn(move || {
            info!(job_id = %job_id_inner, "thread: starting optimization");
            
            // [Fixed] Robust panic handling
            let result = catch_unwind(AssertUnwindSafe(|| {
                keyforge_core::optimize_with_callback(&req, logger)
            }));
            
            let final_res = match result {
                Ok(opt_res) => opt_res.map_err(|e| anyhow::anyhow!(e.to_string())),
                Err(payload) => {
                    let msg = if let Some(s) = payload.downcast_ref::<&str>() {
                        format!("panic: {}", s)
                    } else if let Some(s) = payload.downcast_ref::<String>() {
                        format!("panic: {}", s)
                    } else {
                        "unknown panic type".to_string()
                    };
                    error!(job_id = %job_id_inner, error = %msg, "optimization thread panicked");
                    Err(anyhow::anyhow!(msg))
                }
            };
            
            // Send result back to async world. Ignore error if receiver dropped (timeout).
            let _ = tx.send(final_res);
        })
        .map_err(|e| anyhow::anyhow!("Failed to spawn optimization thread: {}", e))?;

    // Wait for result or timeout
    match tokio::time::timeout(timeout, rx).await {
        Ok(channel_res) => {
            match channel_res {
                Ok(engine_res) => engine_res,
                Err(_) => {
                    // Receiver error: Sender dropped without sending. 
                    // This happens if the thread finishes but fails to send (unlikely) or panic crash happened weirdly.
                    Err(anyhow::anyhow!("optimization thread disconnected unexpectedly"))
                }
            }
        },
        Err(_) => {
            // Timeout occurred. Signal cancellation.
            info!(job_id = %job_id, "optimization timed out, signalling stop");
            stop_flag.store(true, Ordering::SeqCst);
            // We cannot kill the std::thread, so it "leaks" until it checks the flag.
            // But at least we don't hold up the Agent logic.
            Err(anyhow::anyhow!("optimization timed out after {} seconds", timeout_sec))
        }
    }
}
