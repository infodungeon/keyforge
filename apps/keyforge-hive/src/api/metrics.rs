use crate::error::AppResult;
use crate::observability::{get_recent_logs, LogEntry};
use crate::state::AppState;
use axum::{extract::State, response::IntoResponse, Json};
use keyforge_protocol::SystemMetrics;
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub struct SystemStatusResponse {
    pub metrics: SystemMetrics,
    pub logs: Vec<LogEntry>,
}

pub async fn get_metrics(State(_state): State<Arc<AppState>>) -> AppResult<impl IntoResponse> {
    // This endpoint exposes Prometheus metrics (best-effort).
    let body = match crate::observability::get_metrics_handle() {
        Some(handle) => handle.render(),
        None => "# metrics disabled\n".to_string(),
    };

    Ok(body)
}

pub async fn get_system_status(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<SystemStatusResponse>> {
    // 1. Refresh System Monitor
    let (uptime, ram, cpu) = {
        let mut monitor = state.monitor.lock().await;
        monitor.refresh();
        (
            monitor.get_uptime(),
            monitor.get_memory_used(),
            monitor.get_cpu_usage(),
        )
    };

    // 2. Fetch Cluster Stats (Placeholder for now, or simple counts)
    // In a real scenario, we'd query the DB or atomic counters.
    // For now, we'll use the queue depth as a proxy for activity or just 0.
    let active_jobs = 0; // TODO: state.jobs.count_active().await
    let nodes_online = 0; // TODO: state.nodes.count_online().await
    let total_results = 0; // TODO: state.results.count_total().await

    let metrics = SystemMetrics {
        uptime_secs: uptime,
        active_jobs,
        total_results,
        nodes_online,
        total_ops_per_sec: 0.0,
        server_memory_used: ram,
        server_cpu_usage: cpu,
    };

    // 3. Fetch Logs
    let logs = get_recent_logs();

    Ok(Json(SystemStatusResponse { metrics, logs }))
}
