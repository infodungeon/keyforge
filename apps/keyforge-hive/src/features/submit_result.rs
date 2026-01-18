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


use axum::{extract::State, Json};
use keyforge_model::Validator;
use keyforge_protocol::{ResultSubmission, PROTOCOL_VERSION};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::oneshot;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::infra::queue::DbEvent;
use crate::config::DEFAULT_SUBMISSION_EXPIRATION_SECS;

/// VSA Feature: Submit Result
/// Handles result verification, replay protection, and persistence.

#[utoipa::path(
    post,
    path = "/results",
    request_body = ResultSubmission,
    responses(
        (status = 200, description = "Result accepted"),
        (status = 400, description = "Invalid submission")
    ),
    tag = "results"
)]
/// Handles the submission of an optimization result from a compute node.
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ResultSubmission>,
) -> AppResult<String> {
    process_submission(&state, payload).await
}

/// Orchestrates the result submission flow: validation, cryptographic verification, and persistent storage.
async fn process_submission(state: &AppState, payload: ResultSubmission) -> AppResult<String> {
    // Stage 1: Validation
    payload.validate().map_err(AppError::Validation)?;
    validate_submission(state, &payload).await?;

    // Stage 2: Verification (Domain Logic)
    state.verification.verify_submission(&payload).await?;

    // Stage 3: Persistence (Via Queue)
    persist_result(state, payload).await?;

    state.jobs.active_count.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    state.jobs.completed_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    state.monitor.record_op();

    Ok("Accepted".to_string())
}

/// Performs technical validation of the submission, including protocol consistency and replay protection.
async fn validate_submission(state: &AppState, payload: &ResultSubmission) -> AppResult<()> {
    // Protocol Check
    if payload.version != PROTOCOL_VERSION {
        return Err(AppError::Validation("Protocol Mismatch".into()));
    }

    // Expiration Check
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    if now.abs_diff(payload.timestamp) > DEFAULT_SUBMISSION_EXPIRATION_SECS {
        return Err(AppError::Validation("Submission expired".into()));
    }

    // Replay Protection (Task-hive-009: Use Valkey for distributed safety)
    let is_new = state.coordinator.check_and_set_nonce(
        &payload.node_id, 
        payload.nonce, 
        DEFAULT_SUBMISSION_EXPIRATION_SECS as i64
    ).await.map_err(|e| AppError::Any(anyhow::anyhow!("Valkey Error: {e}")))?;

    if !is_new {
        return Err(AppError::Validation("Replay detected".into()));
    }

    Ok(())
}

/// Pushes the verified result to the background write queue for durable persistence.
async fn persist_result(state: &AppState, payload: ResultSubmission) -> AppResult<()> {
    let (tx, rx) = oneshot::channel();
    state
        .queue
        .push(DbEvent::Result {
            job_id: payload.job_id.clone(),
            layout: payload.layout.clone(),
            score: payload.score,
            node_id: payload.node_id.clone(),
            ack: Some(tx),
        })
        .await;

    rx.await.map_err(|_| AppError::Any(anyhow::anyhow!("Persistence failed")))?;
    Ok(())
}
