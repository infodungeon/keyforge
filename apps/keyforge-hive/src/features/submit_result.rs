// apps/keyforge-hive/src/features/submit_result.rs

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

use crate::commands::{handle_command, HiveCommand};
use crate::error::AppResult;
use crate::state::AppState;
use axum::{extract::State, Json};
use keyforge_protocol::ResultSubmission;
use std::sync::Arc;

/// VSA Feature: Submit Result
/// Handles result verification, replay protection, and persistence.

#[utoipa::path(
    post,
    path = "/results",
    request_body = ResultSubmission,
    responses(
        (status = 200, description = "Result accepted"),
        (status = 400, description = "Invalid result signature or score")
    ),
    tag = "results"
)]
/// Handles a result submission from a worker node.
#[tracing::instrument(skip_all)] pub(crate) async fn handle(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ResultSubmission>,
) -> AppResult<Json<()>> {
    handle_command(&state, HiveCommand::SubmitResult(payload)).await?;
    Ok(Json(()))
}
