// apps/keyforge-hive/src/features/list_submissions.rs

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
use serde::Serialize;
use utoipa::ToSchema;
use std::sync::Arc;
use crate::error::AppResult;
use crate::state::AppState;

/// A single entry representing a community layout submission.
#[derive(Serialize, ToSchema, Clone, Debug)]
pub struct SubmissionEntry {
    /// Unique identifier for the submission.
    pub id: i64,
    /// Name of the layout.
    pub name: String,
    /// JSON serialization of the layout configuration.
    pub layout: String,
    /// Name of the author.
    pub author: String,
    /// ISO-8601 formatted timestamp of the submission.
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
/// Handles a request to list the most recent community layout submissions.
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
