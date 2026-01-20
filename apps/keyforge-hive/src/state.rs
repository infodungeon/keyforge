// apps/keyforge-hive/src/state.rs

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

use crate::cache::CompiledEngineCache;
use crate::config::{AppConfig, DEFAULT_BROADCAST_CAPACITY, DEFAULT_MONITOR_INTERVAL_SECS};
use crate::infra::queue::WriteQueue;
use crate::infra::repositories::{
    AuditRepository, JobRepository, NodeRepository, ResultRepository, SubmissionRepository,
    UserRepository,
};
use crate::monitor::{SharedMonitor, SystemMonitor};
use crate::services::job_manager::JobManager;
use crate::services::security::SecurityContext;
use crate::services::verification::VerificationService;
use keyforge_infra::{DistributedCoordinator, ValkeyDistributedCoordinator, ValkeyProvider};
use sqlx::{Pool, Postgres};

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Global application state for the `KeyForge` Hive.
#[derive(Clone, Debug)]
pub struct AppState {
    /// Flag indicating if the required system assets (corpora, etc.) are available.
    pub assets_healthy: Arc<std::sync::atomic::AtomicBool>,
    /// Manager for optimization jobs and life cycle.
    pub jobs: Arc<JobManager>,
    /// Global security context for secrets and keys.
    pub security: Arc<SecurityContext>,
    /// Service for verifying and ranking submitted layouts.
    pub verification: Arc<VerificationService>,
    /// Repository for worker node metadata.
    pub nodes: NodeRepository,
    /// Repository for optimization results.
    pub results: ResultRepository,
    /// Repository for layout submissions.
    pub submissions: SubmissionRepository,
    /// Repository for user management.
    pub users: UserRepository,
    /// Repository for security audit logs.
    pub audit: AuditRepository,
    /// Queue for asynchronous result persistence.
    pub queue: Arc<WriteQueue>,
    /// High-level asset provider (Valkey-backed).
    pub assets: Arc<ValkeyProvider>,
    /// Cache for pre-compiled optimization engines.
    pub engine_cache: Arc<CompiledEngineCache>,
    /// Static Hive configuration loaded from the environment/assets.
    pub config: Arc<AppConfig>,
    /// Real-time system monitoring and metrics.
    pub monitor: SharedMonitor,
    /// Local filesystem path for transient storage.
    pub data_path: PathBuf,
    /// Broadcast channel for real-time events (e.g., TUI updates).
    pub tx: broadcast::Sender<String>,
    /// Coordinator for distributed state (Valkey).
    pub coordinator: Arc<dyn DistributedCoordinator>,
}

impl AppState {
    /// Initializes the `AppState` by connecting to the database and Valkey,
    /// and starting background monitors.
    ///
    /// # Panics
    ///
    /// Panics if the connection to the Coordination Layer (Valkey) fails.
    #[allow(clippy::expect_used)]
    pub async fn new(
        db: Pool<Postgres>,
        data_path: PathBuf,
        server_key: String,
        config: AppConfig,
    ) -> Self {
        let job_repo = JobRepository::new(db.clone());
        let nodes = NodeRepository::new(db.clone());
        let results = ResultRepository::new(db.clone(), config.population_limit);
        let submissions = SubmissionRepository::new(db.clone());
        let users = UserRepository::new(db.clone());
        let audit = AuditRepository::new(db.clone());

        let coordinator: Arc<dyn DistributedCoordinator> = Arc::new(
            ValkeyDistributedCoordinator::new(&config.valkey_url)
                .await
                .expect("Failed to connect to Coordination Layer (Valkey)"),
        );

        let assets = Arc::new(ValkeyProvider::new(coordinator.clone()));

        let config_arc = Arc::new(config.clone());

        let queue = Arc::new(WriteQueue::new(
            results.clone(),
            data_path.clone(),
            config.queue.clone(),
        ));

        let (tx, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);

        // HIVE_SECRET is now enforced by AppConfig
        let security = Arc::new(SecurityContext::new(Some(config.hive_secret), server_key));

        let monitor = Arc::new(SystemMonitor::new());

        // Task-hive-014: Optimized background refresh
        monitor
            .clone()
            .start_background_refresh(DEFAULT_MONITOR_INTERVAL_SECS);

        let jobs = Arc::new(JobManager::new(job_repo.clone(), queue.clone()));
        let engine_cache = Arc::new(CompiledEngineCache::new());

        let verification = Arc::new(VerificationService::new(
            job_repo,
            nodes.clone(),
            assets.clone(),
            engine_cache.clone(),
        ));

        Self {
            assets_healthy: Arc::new(std::sync::atomic::AtomicBool::new(true)),
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
            config: config_arc,
            monitor,
            data_path,
            tx,
            coordinator,
        }
    }
}
