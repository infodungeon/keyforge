// ===== keyforge/crates/keyforge-hive/src/state.rs =====
use crate::cache::{CompiledEngineCache, GlobalAssetCache}; // CHANGED: Added CompiledEngineCache
use crate::config::HiveConfig;
use crate::infra::queue::WriteQueue;
use crate::infra::repositories::{
    AuditRepository, JobRepository, NodeRepository, ResultRepository, SubmissionRepository,
    UserRepository,
};
use crate::monitor::{SharedMonitor, SystemMonitor};
use crate::services::verification::VerificationService;
use moka::sync::Cache;
use sqlx::{Pool, Postgres};
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, Mutex, Notify, Semaphore};

#[derive(Clone)]
pub struct AppState {
    /// Whether startup asset warmup succeeded.
    /// When false, the server is running in degraded mode.
    pub assets_healthy: std::sync::Arc<std::sync::atomic::AtomicBool>,

    pub jobs: JobRepository,
    pub nodes: NodeRepository,
    pub results: ResultRepository,
    pub submissions: SubmissionRepository,
    pub users: UserRepository,
    pub audit: AuditRepository,

    pub verification: Arc<VerificationService>,

    pub queue: Arc<WriteQueue>,
    pub assets: Arc<GlobalAssetCache>,

    // Caches compiled physics engines to avoid re-compiling per request
    pub engine_cache: Arc<CompiledEngineCache>,

    pub config: Arc<HiveConfig>,

    pub api_secret: Option<String>,
    pub api_key_cache: Cache<String, bool>,
    pub nonce_cache: Cache<String, bool>,

    pub server_key: String,

    pub monitor: SharedMonitor,
    pub job_signal: Arc<Notify>,
    pub poll_semaphore: Arc<Semaphore>,

    pub data_path: PathBuf,
    pub tx: broadcast::Sender<String>,
}

impl AppState {
    pub fn new(db: Pool<Postgres>, data_path: PathBuf, server_key: String) -> Self {
        let jobs = JobRepository::new(db.clone());
        let nodes = NodeRepository::new(db.clone());
        let results = ResultRepository::new(db.clone());
        let submissions = SubmissionRepository::new(db.clone());
        let users = UserRepository::new(db.clone());
        let audit = AuditRepository::new(db.clone());

        // Initialize Assets FIRST to load config
        let assets = Arc::new(GlobalAssetCache::new(data_path.clone()));
        let config = assets.load_hive_config();

        // Pass assets to queue for dynamic config access
        let queue = Arc::new(WriteQueue::new(
            results.clone(),
            data_path.clone(),
            assets.clone(),
        ));

        let (tx, _) = broadcast::channel(10000);

        let api_secret = env::var("HIVE_SECRET").ok().filter(|s| !s.is_empty());

        let api_key_cache = Cache::builder()
            .max_capacity(1000)
            .time_to_live(Duration::from_secs(300))
            .build();

        let nonce_cache = Cache::builder()
            .max_capacity(100_000)
            .time_to_live(Duration::from_secs(600))
            .build();

        let monitor = Arc::new(Mutex::new(SystemMonitor::new()));
        let job_signal = Arc::new(Notify::new());
        let poll_semaphore = Arc::new(Semaphore::new(1000));

        // Initialize the Engine Cache for verification performance
        let engine_cache = Arc::new(CompiledEngineCache::new());

        let verification = Arc::new(VerificationService::new(
            jobs.clone(),
            nodes.clone(),
            assets.clone(),
            engine_cache.clone(),
        ));

        Self {
            assets_healthy: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true)),
            jobs,
            nodes,
            results,
            submissions,
            users,
            audit,
            verification,
            queue,
            assets,
            engine_cache,
            config,
            api_secret,
            api_key_cache,
            nonce_cache,
            server_key,
            monitor,
            job_signal,
            poll_semaphore,
            data_path,
            tx,
        }
    }
}
