use axum::{extract::{Path, State}, Json};
use keyforge_protocol::JobStatus;
use std::sync::Arc;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

/// VSA Feature: Get Job Status
/// Returns comprehensive status and statistics for a job.

#[utoipa::path(
    get,
    path = "/jobs/{job_id}/status",
    params(
        ("job_id" = String, Path, description = "Job ID")
    ),
    responses(
        (status = 200, description = "Job status", body = JobStatus)
    ),
    tag = "jobs"
)]
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> AppResult<Json<JobStatus>> {
    // 1. Fetch Status from Database
    let status_row = sqlx::query!("SELECT status FROM jobs WHERE id = $1", job_id)
        .fetch_optional(&state.jobs.pool)
        .await
        .map_err(AppError::Database)?;

    let status = match status_row {
        Some(r) => r.status.unwrap_or_else(|| "unknown".to_string()),
        None => return Err(AppError::NotFound),
    };

    // 2. Fetch Performance Stats
    let (nodes, samples, best_score, best_layout) = state
        .results
        .get_stats(&job_id)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(JobStatus {
        job_id,
        status,
        active_nodes: nodes,
        best_score,
        best_layout,
        total_samples: samples,
    }))
}
