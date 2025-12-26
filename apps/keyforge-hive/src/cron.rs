use crate::infra::repositories::{JobRepository, NodeRepository, ResultRepository};
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{error, info};

pub async fn start_cron_jobs(
    job_repo: Arc<JobRepository>,
    node_repo: Arc<NodeRepository>,
    result_repo: Arc<ResultRepository>,
) {
    // CHANGE: Run every minute to catch dead jobs quickly
    let mut interval = interval(Duration::from_secs(60));
    let mut hour_ticker = 0;

    info!("⏰ Starting background maintenance (Interval: 60s)...");

    loop {
        interval.tick().await;
        hour_ticker += 1;

        // --- 1. THE REAPER (Runs every minute) ---
        // Timeout: 10 minutes, Max Retries: 3
        match job_repo.prune_stale_jobs(10, 3).await {
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
            if let Err(e) = job_repo.prune_old_jobs(30).await {
                error!("Failed to prune old jobs: {}", e);
            }

            // Cleanup inactive nodes (> 15 mins)
            if let Err(e) = node_repo.prune_inactive_nodes(15).await {
                error!("Failed to prune inactive nodes: {}", e);
            }

            // Prune Results (Keep top 1000 per job, delete others older than 7 days)
            match result_repo.prune_old_results(7, 1000).await {
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
