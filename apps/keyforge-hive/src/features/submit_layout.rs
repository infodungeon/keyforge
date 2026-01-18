// apps/keyforge-hive/src/features/submit_layout.rs

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



use axum::{extract::State, Json};
use keyforge_model::LayoutValidator;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use std::sync::Arc;
use tracing::{info, warn};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use keyforge_model::constants::{
    MAX_ID_LEN, MIN_LAYOUT_DATA_LEN, MAX_LAYOUT_DATA_LEN, MIN_LAYOUT_NAME_LEN
};

/// Request payload for submitting a new keyboard layout to the community.
#[derive(Deserialize, ToSchema)]
pub struct LayoutSubmission {
    /// Desired name for the layout.
    pub name: String,
    /// Serialization of the layout configuration.
    pub layout: String,
    /// Author's name or pseudonym.
    pub author: String,
}

/// Response confirming the acceptance of a layout submission.
#[derive(Serialize, ToSchema)]
pub struct SubmissionResponse {
    /// Unique identifier assigned to the submission.
    pub id: i64,
    /// Current processing status of the submission.
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
/// Handles a layout submission request, performing validation and persistent storage.
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LayoutSubmission>,
) -> AppResult<Json<SubmissionResponse>> {
    let clean_name = payload.name.trim();
    let clean_author = payload.author.trim();
    let clean_layout = payload.layout.trim();

    // 1. Validation Logic
    if clean_name.len() < MIN_LAYOUT_NAME_LEN || clean_name.len() > MAX_ID_LEN {
        return Err(AppError::Validation(format!("Name must be {MIN_LAYOUT_NAME_LEN}-{MAX_ID_LEN} chars")));
    }
    if clean_author.len() > MAX_ID_LEN {
        return Err(AppError::Validation(format!("Author name too long (max {MAX_ID_LEN})")));
    }
    
    // Check structure before size for better error messages
    LayoutValidator::validate_structure(clean_layout)
        .map_err(|e| AppError::Validation(e.clone()))?;

    if clean_layout.len() < MIN_LAYOUT_DATA_LEN || clean_layout.len() > MAX_LAYOUT_DATA_LEN {
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
