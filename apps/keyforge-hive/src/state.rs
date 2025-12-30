use crate::cache::{CompiledEngineCache, GlobalAssetCache};
use crate::config::HiveConfig;
use crate::infra::queue::WriteQueue;
use crate::infra::repositories::{
    AuditRepository, JobRepository, NodeRepository, ResultRepository, SubmissionRepository,
    UserRepository,
};
use crate::monitor::{SharedMonitor, SystemMonitor};
use crate::services::job_manager::JobManager;
use crate::services::security::SecurityContext;
use crate::services::verification::VerificationService;
use sqlx::{Pool, Postgres};
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    /// Whether startup asset warmup succeeded.
    /// When false, the server is running in degraded mode.
    pub assets_healthy: std::sync::Arc<std::sync::atomic::AtomicBool>,

    // Sub-systems
    pub jobs: Arc<JobManager>,
    pub security: Arc<SecurityContext>,
    pub verification: Arc<VerificationService>,

    // Repositories (Public for now, but should eventually move into services)
    pub nodes: NodeRepository,
    pub results: ResultRepository,
    pub submissions: SubmissionRepository,
    pub users: UserRepository,
    pub audit: AuditRepository,

    // Infrastructure
    pub queue: Arc<WriteQueue>, // Kept here for legacy access if needed, but JobManager has it too
    pub assets: Arc<GlobalAssetCache>,
    pub engine_cache: Arc<CompiledEngineCache>,
    pub config: Arc<HiveConfig>,
    pub monitor: SharedMonitor,
    pub data_path: PathBuf,
    pub tx: broadcast::Sender<String>,
}

impl AppState {
    pub fn new(db: Pool<Postgres>, data_path: PathBuf, server_key: String) -> Self {
        let job_repo = JobRepository::new(db.clone());
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
        let security = Arc::new(SecurityContext::new(api_secret, server_key));

        let monitor = Arc::new(SystemMonitor::new());

        // Background monitor refresh task
        let monitor_clone = monitor.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
            loop {
                interval.tick().await;
                monitor_clone.refresh().await;
            }
        });

        let jobs = Arc::new(JobManager::new(job_repo.clone(), queue.clone()));

        // Initialize the Engine Cache for verification performance
        let engine_cache = Arc::new(CompiledEngineCache::new());

        let verification = Arc::new(VerificationService::new(
            job_repo,
            nodes.clone(),
            assets.clone(),
            engine_cache.clone(),
        ));

        Self {
            assets_healthy: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true)),
            jobs,
            security,
            verification,
            nodes,
            results,
            submissions,
            users,
            audit,
            queue,
            assets,
            engine_cache,
            config,
            monitor,
            data_path,
            tx,
        }
    }
}
