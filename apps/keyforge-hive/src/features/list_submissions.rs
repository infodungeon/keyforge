use axum::{extract::State, Json};
use serde::Serialize;
use utoipa::ToSchema;
use std::sync::Arc;
use crate::error::AppResult;
use crate::state::AppState;

#[derive(Serialize, ToSchema, Clone)]
pub struct SubmissionEntry {
    pub id: i64,
    pub name: String,
    pub layout: String,
    pub author: String,
    pub date: String,
}

/// VSA Feature: List Submissions
/// Returns the most recent community layout submissions.

#[utoipa::path(
    get,
    path = "/submissions",
    responses(
        (status = 200, description = "Recent submissions", body = [SubmissionEntry])
    ),
    tag = "submissions"
)]
pub async fn handle(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Vec<SubmissionEntry>>> {
    let entries = state
        .submissions
        .get_recent(50)
        .await
        .map_err(crate::error::AppError::Database)?;

    Ok(Json(entries))
}
