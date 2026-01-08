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
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ResultSubmission>,
) -> AppResult<String> {
    process_submission(&state, payload).await
}

async fn process_submission(state: &AppState, payload: ResultSubmission) -> AppResult<String> {
    // Stage 1: Validation
    payload.validate().map_err(AppError::Validation)?;
    validate_submission(&state, &payload)?;

    // Stage 2: Verification (Domain Logic)
    state.verification.verify_submission(&payload).await?;

    // Stage 3: Persistence (Via Queue)
    persist_result(state, payload).await?;

    state.jobs.active_count.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    state.jobs.completed_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    state.monitor.record_op();

    Ok("Accepted".to_string())
}

fn validate_submission(state: &AppState, payload: &ResultSubmission) -> AppResult<()> {
    // Protocol Check
    if payload.version != PROTOCOL_VERSION {
        return Err(AppError::Validation("Protocol Mismatch".into()));
    }

    // Expiration Check (15 min window)
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    if now.abs_diff(payload.timestamp) > 900 {
        return Err(AppError::Validation("Submission expired".into()));
    }

    // Replay Protection
    let nonce_key = format!("{}:{}", payload.node_id, payload.nonce);
    if state.security.nonce_cache.contains_key(&nonce_key) {
        return Err(AppError::Validation("Replay detected".into()));
    }
    state.security.nonce_cache.insert(nonce_key, true);

    Ok(())
}

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
