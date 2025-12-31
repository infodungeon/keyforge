use axum::{extract::State, Json};
use keyforge_model::LayoutValidator;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use std::sync::Arc;
use tracing::{info, warn};
use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[derive(Deserialize, ToSchema)]
pub struct LayoutSubmission {
    pub name: String,
    pub layout: String,
    pub author: String,
}

#[derive(Serialize, ToSchema)]
pub struct SubmissionResponse {
    pub id: i64,
    pub status: String,
}

/// VSA Feature: Submit Layout
/// Validates and persists a community layout submission.

#[utoipa::path(
    post,
    path = "/submissions",
    request_body = LayoutSubmission,
    responses(
        (status = 200, description = "Layout submitted", body = SubmissionResponse),
        (status = 400, description = "Invalid layout data")
    ),
    tag = "submissions"
)]
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LayoutSubmission>,
) -> AppResult<Json<SubmissionResponse>> {
    let clean_name = payload.name.trim();
    let clean_author = payload.author.trim();
    let clean_layout = payload.layout.trim();

    // 1. Validation Logic
    if clean_name.len() < 2 || clean_name.len() > 64 {
        return Err(AppError::Validation("Name must be 2-64 chars".into()));
    }
    if clean_author.len() > 64 {
        return Err(AppError::Validation("Author name too long".into()));
    }
    
    // Check structure before size for better error messages
    LayoutValidator::validate_structure(clean_layout)
        .map_err(|e| AppError::Validation(e.to_string()))?;

    if clean_layout.len() < 10 || clean_layout.len() > 5000 {
        return Err(AppError::Validation("Invalid layout data size".into()));
    }

    // 2. Persistence
    let id = state
        .submissions
        .save(clean_name, clean_layout, clean_author)
        .await
        .map_err(|e| {
            warn!("Database error saving submission: {}", e);
            AppError::Database(e)
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
