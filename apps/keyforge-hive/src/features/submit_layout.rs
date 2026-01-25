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

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::{extract::State, Json};
use keyforge_model::constants::{
    MAX_ID_LEN, MAX_LAYOUT_DATA_LEN, MIN_LAYOUT_DATA_LEN, MIN_LAYOUT_NAME_LEN,
};
use keyforge_model::{LayoutValidator, Validator};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};
use utoipa::ToSchema;

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

impl Validator for LayoutSubmission {
    fn validate(&self) -> Result<(), String> {
        let clean_name = self.name.trim();
        let clean_author = self.author.trim();
        let clean_layout = self.layout.trim();

        if clean_name.len() < MIN_LAYOUT_NAME_LEN || clean_name.len() > MAX_ID_LEN {
            return Err(format!(
                "Name must be {MIN_LAYOUT_NAME_LEN}-{MAX_ID_LEN} chars"
            ));
        }
        if clean_author.len() > MAX_ID_LEN {
            return Err(format!("Author name too long (max {MAX_ID_LEN})"));
        }

        LayoutValidator::validate_structure(clean_layout)?;

        if clean_layout.len() < MIN_LAYOUT_DATA_LEN || clean_layout.len() > MAX_LAYOUT_DATA_LEN {
            return Err("Invalid layout data size".into());
        }
        Ok(())
    }
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
#[tracing::instrument(skip_all, fields(name = %payload.name, author = %payload.author))]
#[tracing::instrument(skip_all)]
pub(crate) async fn handle(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LayoutSubmission>,
) -> AppResult<Json<SubmissionResponse>> {
    // 1. Validation Logic
    payload.validate().map_err(AppError::Validation)?;

    let clean_name = payload.name.trim();
    let clean_author = payload.author.trim();
    let clean_layout = payload.layout.trim();

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
