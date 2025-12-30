use axum::extract::{Path, State};
use std::sync::Arc;
use tracing::info;
use crate::error::AppResult;
use crate::state::AppState;

/// VSA Feature: Cancel Job
/// Terminates a job and notifies active workers.

#[utoipa::path(
    delete,
    path = "/jobs/{job_id}",
    params(
        ("job_id" = String, Path, description = "Job ID")
    ),
    responses(
        (status = 200, description = "Job cancelled")
    ),
    tag = "jobs"
)]
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> AppResult<String> {
    state
        .jobs
        .repo
        .cancel(&job_id)
        .await
        .map_err(crate::error::AppError::Database)?;

    let _ = state.tx.send(format!("CANCEL:{}", job_id));

    info!("🛑 Cancelled Job: {}", &job_id[0..8]);
    Ok("Job cancelled".to_string())
}
