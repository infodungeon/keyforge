// apps/keyforge-hive/src/features/register_job.rs

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
use keyforge_protocol::{JobRequest, JobResponse};
use std::sync::Arc;

use crate::commands::{handle_command, CommandResponse, HiveCommand};
use crate::error::AppResult;
use crate::state::AppState;

#[utoipa::path(
    post,
    path = "/jobs",
    request_body = JobRequest,
    responses(
        (status = 200, description = "Job registered successfully", body = JobResponse),
        (status = 400, description = "Invalid request parameters")
    ),
    tag = "jobs"
)]
/// Handles a request to register a new optimization job.
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<JobRequest>,
) -> AppResult<Json<JobResponse>> {
    // Reified Action: Intent is clearly separated from execution
    let response = handle_command(&state, HiveCommand::RegisterJob(payload)).await?;

    if let CommandResponse::JobRegistered(res) = response {
        Ok(Json(res))
    } else {
        Err(crate::error::AppError::Any(anyhow::anyhow!(
            "Unexpected command response"
        )))
    }
}
