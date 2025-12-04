use crate::state::AppState;
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};

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

// NEW: For listing
#[derive(Serialize, Clone)]
pub struct SubmissionEntry {
    pub id: i64,
    pub name: String,
    pub layout: String,
    pub author: String,
    pub date: String,
}

pub async fn submit_layout(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LayoutSubmission>,
) -> Result<Json<SubmissionResponse>, (StatusCode, String)> {
    let clean_name = payload.name.trim();
    let clean_author = payload.author.trim();
    let clean_layout = payload.layout.trim();

    if clean_name.len() < 2 || clean_name.len() > 64 {
        return Err((StatusCode::BAD_REQUEST, "Name must be 2-64 chars".into()));
    }
    if clean_author.len() > 64 {
        return Err((StatusCode::BAD_REQUEST, "Author name too long".into()));
    }
    if clean_layout.len() < 10 || clean_layout.len() > 5000 {
        return Err((StatusCode::BAD_REQUEST, "Invalid layout data size".into()));
    }

    let id = state
        .store
        .save_submission(clean_name, clean_layout, clean_author)
        .await
        .map_err(|e| {
            warn!("Database error saving submission: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error".into())
        })?;

    info!(
        "📨 Community Submission [#{}] '{}' by {}",
        id, clean_name, clean_author
    );

    Ok(Json(SubmissionResponse {
        id,
        status: "received".to_string(),
    }))
}

// NEW: GET handler
pub async fn list_submissions(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<SubmissionEntry>>, (StatusCode, String)> {
    let entries = state
        .store
        .get_recent_submissions(50) // Hard limit 50 for now
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(entries))
}
