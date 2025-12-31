use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::{extract::State, Json};
use serde::Serialize;
use std::sync::Arc;
use tracing::info;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct AdminStatsResponse {
    active_jobs: i64,
    total_results: i64,
    nodes_online: i64,
    total_ops_per_sec: f32,
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
    let nodes_online = state
        .nodes
        .count_recent()
        .await
        .map_err(AppError::Database)?;
    let total_ops_per_sec = state.nodes.sum_ops().await.map_err(AppError::Database)?;
    let queue_depth = state.queue.current_depth().await;

    Ok(Json(AdminStatsResponse {
        active_jobs,
        total_results,
        nodes_online,
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
pub async fn reload_config(State(_state): State<Arc<AppState>>) -> AppResult<String> {
    info!("⚙️ Admin requested config reload (not yet implemented)");
    Ok("Config reload initiated".to_string())
}

#[derive(Serialize, ToSchema)]
pub struct FullBackup {
    keyboards: Vec<serde_json::Value>,
    jobs: Vec<serde_json::Value>,
    results_sample: Vec<serde_json::Value>,
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
pub async fn clear_cache(State(state): State<Arc<AppState>>) -> AppResult<String> {
    info!("🧹 Admin requested global cache invalidation");
    state.assets.invalidate_all();
    state.engine_cache.invalidate_all();
    Ok("Cache cleared successfully".to_string())
}
