// apps/keyforge-hive/src/api/metrics.rs

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::{extract::State, Json};
use keyforge_protocol::SystemMetrics;
use std::sync::Arc;

/// GET /metrics
/// Returns global performance and health metrics for the Hive.
#[utoipa::path(
    get,
    path = "/metrics",
    responses(
        (status = 200, description = "Current system metrics", body = SystemMetrics)
    ),
    tag = "system"
)]
#[tracing::instrument(skip_all)]
pub(crate) async fn get_metrics(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<SystemMetrics>> {
    // 1. Fetch Node stats from Coordinator
    let (total_nodes, total_ips) = state
        .coordinator
        .get_cluster_stats()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // 2. Fetch Job stats from Database
    let (active_jobs, pending_jobs, total_results) = state
        .jobs
        .repo
        .count_active()
        .await
        .map(|c| {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let active = c as usize;
            (active, 0, 0)
        })
        .unwrap_or((0, 0, 0));

    // 3. Fetch Local Server Stats
    let (ram, cpu) = fetch_server_stats();

    // 4. Count Online Nodes
    let nodes_online = state.coordinator.count_active_nodes().await.unwrap_or(0);

    #[allow(clippy::cast_sign_loss)]
    let total_results_u64 = total_results as u64;

    Ok(Json(SystemMetrics {
        total_nodes,
        total_ips,
        active_jobs,
        pending_jobs,
        completed_jobs: 0,
        total_results: total_results_u64,
        uptime_secs: 0, // TODO: Track start_time in AppState
        nodes_online,
        total_ops_per_sec: total_ips,
        server_memory_used: ram,
        server_cpu_usage: cpu,
    }))
}

fn fetch_server_stats() -> (u64, f32) {
    (0, 0.0)
}
