// apps/keyforge-hive/src/state.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


use crate::cache::CompiledEngineCache;
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
use keyforge_infra::{DistributedCoordinator, ValkeyProvider};
use sqlx::{Pool, Postgres};
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub assets_healthy: std::sync::Arc<std::sync::atomic::AtomicBool>,
    pub jobs: Arc<JobManager>,
    pub security: Arc<SecurityContext>,
    pub verification: Arc<VerificationService>,
    pub nodes: NodeRepository,
    pub results: ResultRepository,
    pub submissions: SubmissionRepository,
    pub users: UserRepository,
    pub audit: AuditRepository,
    pub queue: Arc<WriteQueue>,
    pub assets: Arc<ValkeyProvider>,
    pub engine_cache: Arc<CompiledEngineCache>,
    pub config: Arc<HiveConfig>,
    pub monitor: SharedMonitor,
    pub data_path: PathBuf,
    pub tx: broadcast::Sender<String>,
    pub coordinator: Arc<DistributedCoordinator>,
}

impl AppState {
    pub async fn new(db: Pool<Postgres>, data_path: PathBuf, server_key: String) -> Self {
        let job_repo = JobRepository::new(db.clone());
        let nodes = NodeRepository::new(db.clone());
        let results = ResultRepository::new(db.clone());
        let submissions = SubmissionRepository::new(db.clone());
        let users = UserRepository::new(db.clone());
        let audit = AuditRepository::new(db.clone());

        let valkey_url = env::var("KEYFORGE_VALKEY_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

        let coordinator = Arc::new(
            DistributedCoordinator::new(&valkey_url)
                .await
                .expect("Failed to connect to Coordination Layer (Valkey)"),
        );

        let assets = Arc::new(ValkeyProvider::new(coordinator.clone()));
        
        // FIX: Use generic loader with string "hive" (for hive.json)
        let config: Arc<HiveConfig> = assets.load_config_asset("hive").await;

        let queue = Arc::new(WriteQueue::new(
            results.clone(),
            data_path.clone(),
            assets.clone(),
        ));

        let (tx, _) = broadcast::channel(10000);
        let api_secret = env::var("HIVE_SECRET").ok().filter(|s| !s.is_empty());
        let security = Arc::new(SecurityContext::new(api_secret, server_key));
        let monitor = Arc::new(SystemMonitor::new());

        let monitor_clone = monitor.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
            loop {
                interval.tick().await;
                monitor_clone.refresh().await;
            }
        });

        let jobs = Arc::new(JobManager::new(job_repo.clone(), queue.clone()));
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
            coordinator,
        }
    }
}
