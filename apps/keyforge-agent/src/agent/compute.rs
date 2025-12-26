use anyhow::{Context, Result};
use keyforge_adapter::conversion;
use keyforge_core::EngineRequest;
use keyforge_infra::{AssetManager, UserRepo};
use keyforge_model::loader::AssetLoader;
use keyforge_model::OptimizationResult;
use keyforge_protocol::JobConfig;
use keyforge_protocol::Validator;
use std::future::Future;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::info;

pub trait AssetSyncer {
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

pub async fn prepare_assets<S: AssetSyncer>(
    syncer: &S,
    config: &JobConfig,
) -> Result<(String, String)> {
    syncer.sync_assets(config).await
}

pub struct PreparedJob {
    pub req: EngineRequest,
    pub registry: keyforge_protocol::keycodes::KeycodeRegistry,
    // Kept for deterministic recalculation and reporting
    pub keyboard: std::sync::Arc<keyforge_model::Keyboard>,
    pub corpus: std::sync::Arc<keyforge_model::Corpus>,
    pub rubric: std::sync::Arc<keyforge_model::Rubric>,
}

pub fn create_engine_request(
    loader: Box<dyn AssetLoader>,
    data_root: PathBuf,
    config: &JobConfig,
    cost_filename: &str,
    _corpus_root: &str,
) -> Result<PreparedJob> {
    config
        .weights
        .validate()
        .map_err(|e| anyhow::anyhow!("invalid weights: {}", e))?;
    config
        .params
        .validate()
        .map_err(|e| anyhow::anyhow!("invalid params: {}", e))?;
    config
        .definition
        .geometry
        .validate()
        .map_err(|e| anyhow::anyhow!("invalid geometry: {}", e))?;

    let user_data = UserRepo::new(data_root.clone());
    let kb_name = &config.definition.meta.name;
    let safe_kb_name = keyforge_infra::sanitize_filename(kb_name);

    user_data
        .save_keyboard_definition(&safe_kb_name, &config.definition)
        .context("failed to save keyboard definition")?;

    // Diversity Pick: Randomly select a parent layout if available
    let start_layout = if !config.parents.is_empty() {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        config.parents.choose(&mut rng).cloned()
    } else {
        None
    };

    // Load all assets needed to build an engine request
    let definition = loader
        .load_keyboard(&safe_kb_name)
        .context("failed to load keyboard definition")?;

    let corpus = loader
        .load_corpus(&config.corpora)
        .context("failed to load corpus")?;

    let raw_cost = loader
        .load_cost_matrix(cost_filename)
        .context("failed to load cost matrix")?;

    let registry = loader
        .load_keycodes("keycodes.json")
        .unwrap_or_else(|_| keyforge_protocol::keycodes::KeycodeRegistry::new_with_defaults());

    let keyboard = conversion::to_domain_keyboard(&definition.geometry);
    let rubric = conversion::to_domain_rubric(&config.weights);
    let cost_overrides = conversion::resolve_cost_matrix(&raw_cost.entries, &definition.geometry);

    let pinned_keys =
        conversion::resolve_constraints(&config.pinned_keys, keyboard.keys.len(), &registry)
            .map_err(|e| anyhow::anyhow!(e))?;

    let search_config = conversion::to_domain_config(&config.params, 42);

    let initial_layout = match start_layout {
        Some(layout_str) => Some(
            conversion::parse_layout_string(&layout_str, keyboard.keys.len(), &registry)
                .map_err(|e| anyhow::anyhow!(e))?,
        ),
        None => None,
    };

    let keyboard = std::sync::Arc::new(keyboard);
    let corpus = std::sync::Arc::new(corpus);
    let rubric = std::sync::Arc::new(rubric);

    let req = EngineRequest {
        keyboard: keyboard.clone(),
        corpus: corpus.clone(),
        rubric: rubric.clone(),
        config: search_config,
        initial_layout,
        pinned_keys,
        cost_overrides,
    };

    Ok(PreparedJob {
        req,
        registry,
        keyboard,
        corpus,
        rubric,
    })
}

pub async fn run_optimization(
    req: EngineRequest,
    job_id: String,
    stop_flag: Arc<AtomicBool>,
    limiter: Arc<Semaphore>,
) -> Result<OptimizationResult> {
    // P1 FIX: Acquire permit before spawning heavy blocking task
    let _permit = limiter
        .acquire()
        .await
        .map_err(|_| anyhow::anyhow!("Semaphore closed"))?;

    info!(job_id = %job_id, "starting optimization loop");

    let logger = crate::agent::telemetry::WorkerLogger {
        stop_flag: stop_flag.clone(),
        job_id: job_id.clone(),
    };

    let timeout = tokio::time::Duration::from_secs(3600);

    let handle = tokio::task::spawn_blocking(move || {
        let result = catch_unwind(AssertUnwindSafe(|| {
            keyforge_core::optimize_with_callback(&req, logger)
        }));

        match result {
            Ok(opt_res) => Ok(opt_res),
            Err(err) => {
                let msg = if let Some(s) = err.downcast_ref::<&str>() {
                    format!("panic: {}", s)
                } else if let Some(s) = err.downcast_ref::<String>() {
                    format!("panic: {}", s)
                } else {
                    "unknown panic".to_string()
                };
                Err(anyhow::anyhow!(msg))
            }
        }
    });

    match tokio::time::timeout(timeout, handle).await {
        Ok(spawn_res) => match spawn_res {
            Ok(optimization_res) => optimization_res,
            Err(join_err) => {
                if join_err.is_panic() {
                    Err(anyhow::anyhow!("optimization task panicked (join error)"))
                } else {
                    Err(anyhow::anyhow!("optimization task cancelled"))
                }
            }
        },
        Err(_) => {
            stop_flag.store(true, std::sync::atomic::Ordering::SeqCst);
            Err(anyhow::anyhow!("optimization timed out after 1 hour"))
        }
    }
}
