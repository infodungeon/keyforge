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

use crate::config::DEFAULT_SUBMISSION_EXPIRATION_SECS;
use crate::error::{AppError, AppResult};
use crate::infra::queue::PersistedRecord;
use crate::state::AppState;
use axum::{extract::State, Json};
use keyforge_model::Validator;
use keyforge_protocol::{ResultSubmission, PROTOCOL_VERSION};
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
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ResultSubmission>,
) -> AppResult<Json<()>> {
    validate_submission(&state, &payload).await?;
    persist_result(&state, payload)?;
    Ok(Json(()))
}

/// Validates the submission protocol, signature, score, and replay protection.
async fn validate_submission(state: &AppState, payload: &ResultSubmission) -> AppResult<()> {
    if payload.version != PROTOCOL_VERSION {
        return Err(AppError::Validation(format!(
            "Protocol Mismatch. Server: v{PROTOCOL_VERSION}, Client: v{}",
            payload.version
        )));
    }

    payload.validate().map_err(AppError::Validation)?;
    state.verification.verify_submission(payload).await?;

    // Replay Protection (Task-hive-009: Use Valkey for distributed safety)
    #[allow(clippy::cast_possible_wrap)]
    let is_new = state
        .coordinator
        .check_and_set_nonce(
            &payload.node_id,
            payload.nonce,
            DEFAULT_SUBMISSION_EXPIRATION_SECS as i64,
        )
        .await
        .map_err(|e| AppError::Any(anyhow::anyhow!("Valkey Error: {e}")))?;

    if !is_new {
        return Err(AppError::Validation("Replay detected".into()));
    }

    Ok(())
}

/// Pushes the verified result to the background write queue for durable persistence.
fn persist_result(state: &AppState, payload: ResultSubmission) -> AppResult<()> {
    state
        .queue
        .push(PersistedRecord {
            job_id: payload.job_id,
            layout: payload.layout,
            score: payload.score,
            node_id: payload.node_id,
        })
        .map_err(|e| {
            if e == "Queue full" {
                AppError::ServiceUnavailable("Persistence queue full".into())
            } else {
                AppError::Any(anyhow::anyhow!("Persistence failed: {e}"))
            }
        })?;

    Ok(())
}
