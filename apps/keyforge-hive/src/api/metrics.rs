// apps/keyforge-hive/src/api/metrics.rs

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


use crate::error::AppResult;
use crate::observability::{get_recent_logs, LogEntry};
use crate::state::AppState;
use axum::{extract::State, response::IntoResponse, Json};
use keyforge_protocol::SystemMetrics;
use serde::Serialize;
use std::sync::Arc;

/// Response payload combining system metrics and recent logs for the monitoring TUI.
#[derive(Serialize)]
pub struct SystemStatusResponse {
    /// Comprehensive system performance metrics.
    pub metrics: SystemMetrics,
    /// Collection of recent log entries.
    pub logs: Vec<LogEntry>,
}

/// Renders Prometheus-formatted metrics for scraping.
pub async fn get_metrics(State(_state): State<Arc<AppState>>) -> AppResult<impl IntoResponse> {
    let body = match crate::observability::get_metrics_handle() {
        Some(handle) => handle.render(),
        None => "# metrics disabled\n".to_string(),
    };
    Ok(body)
}

/// Retrieves the complete system status, including cluster-wide metrics and recent logs.
pub async fn get_system_status(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<SystemStatusResponse>> {
    let uptime = state.monitor.get_uptime();
    let ram = state.monitor.get_memory_used();
    let cpu = state.monitor.get_cpu_usage();
    
    // Local OPS (Server-side verification throughput)
    // let ops_per_sec = state.monitor.get_ops_per_sec();

    #[allow(clippy::cast_possible_wrap)]
    let active_jobs = state.jobs.active_count.load(std::sync::atomic::Ordering::Relaxed) as i64;
    
    // FETCH DISTRIBUTED STATS (Valkey)
    let (nodes_online, total_ops_per_sec) = state
        .coordinator
        .get_cluster_stats()
        .await
        .unwrap_or((0, 0.0));
    
    #[allow(clippy::cast_possible_wrap)]
    let total_results = state.jobs.completed_count.load(std::sync::atomic::Ordering::Relaxed) as i64;

    let metrics = SystemMetrics {
        uptime_secs: uptime,
        active_jobs,
        total_results,
        #[allow(clippy::cast_possible_wrap)]
        nodes_online: nodes_online as i64,
        total_ops_per_sec, // Cluster-wide OPS
        server_memory_used: ram,
        server_cpu_usage: cpu,
    };

    let logs = get_recent_logs();

    Ok(Json(SystemStatusResponse { metrics, logs }))
}
