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
