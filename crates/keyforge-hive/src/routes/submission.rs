use crate::state::AppState;
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

#[derive(Deserialize)]
pub struct LayoutSubmission {
    pub name: String,
    pub layout: String,
    pub author: String,
}

#[derive(Serialize)]
pub struct SubmissionResponse {
    pub id: i64,
    pub status: String,
}

pub async fn submit_layout(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LayoutSubmission>,
) -> Result<Json<SubmissionResponse>, (StatusCode, String)> {
    if payload.layout.len() < 10 {
        return Err((StatusCode::BAD_REQUEST, "Layout too short".into()));
    }

    let id = state
        .store
        .save_submission(&payload.name, &payload.layout, &payload.author)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    info!(
        "📨 Community Submission: '{}' by {}",
        payload.name, payload.author
    );

    Ok(Json(SubmissionResponse {
        id,
        status: "received".to_string(),
    }))
}
