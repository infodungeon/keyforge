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
use tracing::info;
use rand::seq::SliceRandom;
use crate::models::SharedTelemetry;

/// A trait for types that can synchronize assets required for an optimization job.
pub trait AssetSyncer {
    /// Syncs assets for the given job configuration, returning the cost matrix filename
    /// and the corpora bundle directory name.
    fn sync_assets(
        &self,
        config: &JobConfig,
    ) -> impl Future<Output = Result<(String, String)>> + Send;
}

impl AssetSyncer for AssetManager {
    async fn sync_assets(&self, config: &JobConfig) -> Result<(String, String)> {
        if config.corpora.len() > 10 {
            return Err(anyhow::anyhow!("Too many corpora (limit 10)"));
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
) -> Result<(String, String)> {
    syncer.sync_assets(config).await
}

/// Represents a job that has been hydrated with all necessary domain models
/// and is ready to be passed to the optimization engine.
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
///
/// This involves mapping the protocol-level `JobConfig` to domain-specific models.
pub async fn create_engine_request(
    loader: Box<dyn AssetLoader>,
    data_root: PathBuf,
    config: &JobConfig,
    cost_filename: &str,
    _corpus_root: &str,
) -> Result<PreparedJob> {
    config.weights.validate().map_err(|e| anyhow::anyhow!("invalid weights: {}", e))?;
    config.params.validate().map_err(|e| anyhow::anyhow!("invalid params: {}", e))?;
    config.definition.geometry.validate().map_err(|e| anyhow::anyhow!("invalid geometry: {}", e))?;

    let user_data = UserRepo::new(data_root.clone());
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

    let definition = loader.load_keyboard(&safe_kb_name).await.context("failed to load keyboard definition")?;

    let domain_corpora: Vec<keyforge_model::config::CorpusSource> = config.corpora
        .iter()
        .map(conversion::to_domain_corpus_source)
        .collect();

    let corpus = loader.load_corpus(&domain_corpora).await.context("failed to load corpus")?;
    let raw_cost = loader.load_cost_matrix(cost_filename).await.context("failed to load cost matrix")?;
    let registry = loader.load_keycodes("keycodes.json").await.unwrap_or_else(|_| Arc::new(keyforge_model::keycodes::KeycodeRegistry::new_with_defaults()));

    let keyboard = keyforge_model::Keyboard::new(definition.geometry.keys.clone(), definition.geometry.home_row)
        .map_err(|e| anyhow::anyhow!("Invalid keyboard definition: {}", e))?;
    
    let rubric = conversion::to_domain_rubric(&config.weights);
    let cost_overrides = raw_cost.resolve(&definition.geometry);
    let pinned_keys = conversion::resolve_constraints(&config.pinned_keys, keyboard.keys.len(), &registry).map_err(|e| anyhow::anyhow!(e))?;
    let search_config = conversion::to_domain_config(&config.params, 42);

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
///
/// This function wraps the blocking `keyforge_core` engine call, provides a stop flag
/// for cancellation, and handles telemetry reporting/concurrency limiting.
pub async fn run_optimization(
    req: EngineRequest,
    job_id: String,
    stop_flag: Arc<AtomicBool>,
    limiter: Arc<Semaphore>,
    telemetry: SharedTelemetry,
) -> Result<OptimizationResult> {
    // Acquire permit to respect core limits (even if serial, this prepares for async)
    let _permit = limiter.acquire().await.map_err(|_| anyhow::anyhow!("Semaphore closed"))?;

    info!(job_id = %job_id, "starting optimization loop");

    let logger = crate::agent::telemetry::WorkerLogger {
        stop_flag: stop_flag.clone(),
        job_id: job_id.clone(),
        telemetry: telemetry.clone(),
    };

    let timeout = tokio::time::Duration::from_secs(3600);
    let job_id_inner = job_id.clone();

    let handle = tokio::task::spawn_blocking(move || {
        info!(job_id = %job_id_inner, "task: starting blocking optimization");
        let result = catch_unwind(AssertUnwindSafe(|| {
            keyforge_core::optimize_with_callback(&req, logger)
        }));
        
        info!(
            job_id = %job_id_inner, 
            is_panic = result.is_err(), 
            "task: blocking optimization returned"
        );
        result
    });

    let result = match tokio::time::timeout(timeout, handle).await {
        Ok(spawn_res) => match spawn_res {
            Ok(panic_res) => match panic_res {
                Ok(core_res) => core_res.map_err(|e| anyhow::anyhow!(e.to_string())),
                Err(panic_payload) => {
                    let msg = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                        format!("panic: {}", s)
                    } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                        format!("panic: {}", s)
                    } else {
                        "unknown panic".to_string()
                    };
                    Err(anyhow::anyhow!(msg))
                }
            },
            Err(join_err) => {
                if join_err.is_panic() {
                    Err(anyhow::anyhow!("optimization task panicked (join error)"))
                } else {
                    Err(anyhow::anyhow!("optimization task cancelled"))
                }
            }
        },
        Err(_) => {
            stop_flag.store(true, Ordering::SeqCst);
            Err(anyhow::anyhow!("optimization timed out after 1 hour"))
        }
    };

    let final_stop = stop_flag.load(Ordering::SeqCst);
    info!(
        job_id = %job_id, 
        stopped = final_stop, 
        "run_optimization: final stop_flag check"
    );

    if final_stop {
        return Err(anyhow::anyhow!("optimization cancelled"));
    }

    result
}
