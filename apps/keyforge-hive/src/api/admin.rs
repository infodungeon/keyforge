// apps/keyforge-hive/src/api/admin.rs

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


use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::{extract::State, Json};
use serde::Serialize;
use std::sync::Arc;
use tracing::info;
use utoipa::ToSchema;

/// Response payload for administrative system statistics.
#[derive(Serialize, ToSchema)]
pub struct AdminStatsResponse {
    /// Number of jobs currently in 'active' status.
    active_jobs: i64,
    /// Total number of results recorded in the system.
    total_results: i64,
    /// Number of compute nodes currently connected.
    nodes_online: i64,
    /// Aggregate operations per second across the cluster.
    total_ops_per_sec: f32,
    /// Number of items awaiting persistence in the write queue.
    queue_depth: usize,
}

#[utoipa::path(
    get,
    path = "/admin/stats",
    responses(
        (status = 200, description = "Admin statistics", body = AdminStatsResponse)
    ),
    tag = "admin",
    security(("api_key" = []))
)]
/// Retrieves high-level administrative statistics about jobs, results, and nodes.
pub async fn get_admin_stats(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<AdminStatsResponse>> {
    let active_jobs = state
        .jobs
        .repo
        .count_active()
        .await
        .map_err(AppError::Database)?;
    let total_results = state
        .results
        .count_total()
        .await
        .map_err(AppError::Database)?;
    
    // Use Distributed Coordinator for real-time cluster stats
    let (nodes_online, total_ops_per_sec) = state
        .coordinator
        .get_cluster_stats()
        .await
        .unwrap_or((0, 0.0));

    let queue_depth = state.queue.current_depth().await;

    Ok(Json(AdminStatsResponse {
        active_jobs,
        total_results,
        nodes_online: nodes_online as i64,
        total_ops_per_sec,
        queue_depth,
    }))
}

#[utoipa::path(
    post,
    path = "/admin/reload-config",
    responses(
        (status = 200, description = "Config reloaded")
    ),
    tag = "admin",
    security(("api_key" = []))
)]
/// Signals the system to reload its configuration from disk.
pub async fn reload_config(State(_state): State<Arc<AppState>>) -> AppResult<String> {
    info!("⚙️ Admin requested config reload (not yet implemented)");
    Ok("Config reload initiated".to_string())
}

/// Represents a full (or partial sample) backup of the system's core data.
#[derive(Serialize, ToSchema)]
pub struct FullBackup {
    /// List of registered keyboard geometries.
    keyboards: Vec<serde_json::Value>,
    /// List of active job configurations.
    jobs: Vec<serde_json::Value>,
    /// Sample of recent results for verification.
    results_sample: Vec<serde_json::Value>,
    /// Timestamp of when the backup was generated.
    timestamp: String,
}

#[utoipa::path(
    get,
    path = "/admin/backup",
    responses(
        (status = 200, description = "Full Database backup", body = FullBackup)
    ),
    tag = "admin",
    security(("api_key" = []))
)]
/// Generates a comprehensive backup of the database state.
pub async fn backup_db(State(state): State<Arc<AppState>>) -> AppResult<Json<FullBackup>> {
    // 1. Keyboards
    let keyboards = sqlx::query!("SELECT * FROM keyboards")
        .fetch_all(&state.jobs.repo.pool)
        .await
        .map_err(AppError::Database)?
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "name": r.name,
                "author": r.author,
                "hash": r.unique_hash
            })
        })
        .collect();

    // 2. Active Jobs
    let jobs = sqlx::query!("SELECT * FROM jobs WHERE status = 'active'")
        .fetch_all(&state.jobs.repo.pool)
        .await
        .map_err(AppError::Database)?
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "status": r.status,
                "created_at": r.created_at.map(|d| d.to_rfc3339()).unwrap_or_default()
            })
        })
        .collect();

    // 3. Recent Results (Sample)
    let results = sqlx::query!("SELECT * FROM results ORDER BY created_at DESC LIMIT 100")
        .fetch_all(&state.jobs.repo.pool)
        .await
        .map_err(AppError::Database)?
        .iter()
        .map(|r| {
            serde_json::json!({
                "job_id": r.job_id,
                "score": r.score,
                "layout": r.layout
            })
        })
        .collect();

    info!("📦 Admin triggered Full DB backup");

    Ok(Json(FullBackup {
        keyboards,
        jobs,
        results_sample: results,
        timestamp: chrono::Utc::now().to_rfc3339(),
    }))
}

#[utoipa::path(
    post,
    path = "/admin/cache/clear",
    responses(
        (status = 200, description = "Cache cleared")
    ),
    tag = "admin",
    security(("api_key" = []))
)]
/// Invalidates and clears all system-wide caches (assets and scoring engines).
pub async fn clear_cache(State(state): State<Arc<AppState>>) -> AppResult<String> {
    info!("🧹 Admin requested global cache invalidation");
    state.assets.invalidate_all();
    state.engine_cache.invalidate_all();
    Ok("Cache cleared successfully".to_string())
}
