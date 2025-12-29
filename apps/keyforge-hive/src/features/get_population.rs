use axum::{extract::{Path, State}, Json};
use keyforge_protocol::PopulationResponse;
use std::sync::Arc;
use crate::error::AppResult;
use crate::state::AppState;

/// VSA Feature: Get Population
/// Returns the current population of results for a specific job.

#[utoipa::path(
    get,
    path = "/jobs/{job_id}/population",
    params(
        ("job_id" = String, Path, description = "Job ID")
    ),
    responses(
        (status = 200, description = "Current population", body = PopulationResponse)
    ),
    tag = "jobs"
)]
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> AppResult<Json<PopulationResponse>> {
    let layouts = state
        .results
        .get_population(&job_id)
        .await
        .map_err(crate::error::AppError::Database)?;

    Ok(Json(PopulationResponse { layouts }))
}
