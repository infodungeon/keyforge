// apps/keyforge-hive/src/cron.rs

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


use crate::infra::repositories::{JobRepository, NodeRepository, ResultRepository};
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{error, info};

// --- Defaults ---
pub const DEFAULT_CRON_INTERVAL_SECS: u64 = 60;
pub const DEFAULT_ZOMBIE_TIMEOUT_MINS: i32 = 70;
pub const DEFAULT_ZOMBIE_MAX_RETRIES: i32 = 3;
pub const DEFAULT_PRUNE_OLD_JOBS_DAYS: i32 = 30;
pub const DEFAULT_PRUNE_INACTIVE_NODES_MINS: i32 = 15;
pub const DEFAULT_PRUNE_RESULTS_DAYS: i32 = 7;
pub const DEFAULT_PRUNE_RESULTS_KEEP_COUNT: i32 = 1000;

/// Spawns background maintenance tasks that run periodically.
///
/// This includes the "Zombie Reaper" for stuck jobs, old job cleanup,
/// node pruning, and result database maintenance.
pub async fn start_cron_jobs(
    job_repo: Arc<JobRepository>,
    node_repo: Arc<NodeRepository>,
    result_repo: Arc<ResultRepository>,
) {
    // CHANGE: Run every minute to catch dead jobs quickly
    let mut interval = interval(Duration::from_secs(DEFAULT_CRON_INTERVAL_SECS));
    let mut hour_ticker = 0;

    info!("⏰ Starting background maintenance (Interval: {}s)...", DEFAULT_CRON_INTERVAL_SECS);

    loop {
        interval.tick().await;
        hour_ticker += 1;

        // --- 1. THE REAPER (Runs every minute) ---
        match job_repo.prune_stale_jobs(DEFAULT_ZOMBIE_TIMEOUT_MINS, DEFAULT_ZOMBIE_MAX_RETRIES).await {
            Ok(count) if count > 0 => {
                tracing::warn!("💀 Zombie Reaper: Reset {} stuck jobs.", count);
            }
            Err(e) => tracing::error!("Failed to prune stale jobs: {}", e),
            _ => {} // No zombies found, silence is golden
        }

        // --- 2. HOURLY TASKS (Run every 60 ticks) ---
        if hour_ticker >= 60 {
            hour_ticker = 0;
            info!("🧹 Running hourly maintenance...");

            // Cleanup old jobs (cancelled/stale > 30 days)
            if let Err(e) = job_repo.prune_old_jobs(DEFAULT_PRUNE_OLD_JOBS_DAYS).await {
                error!("Failed to prune old jobs: {}", e);
            }

            // Cleanup inactive nodes (> 15 mins)
            if let Err(e) = node_repo.prune_inactive_nodes(DEFAULT_PRUNE_INACTIVE_NODES_MINS).await {
                error!("Failed to prune inactive nodes: {}", e);
            }

            // Prune Results (Keep top 1000 per job, delete others older than 7 days)
            match result_repo.prune_old_results(DEFAULT_PRUNE_RESULTS_DAYS, DEFAULT_PRUNE_RESULTS_KEEP_COUNT).await {
                Ok(count) => {
                    if count > 0 {
                        info!("🧹 Pruned {} old results.", count);
                    }
                }
                Err(e) => error!("Failed to prune results: {}", e),
            }
        }
    }
}
