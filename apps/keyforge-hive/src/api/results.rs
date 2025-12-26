use crate::error::{AppError, AppResult};
use crate::infra::queue::DbEvent;
use crate::state::AppState;
use axum::{extract::State, Json};
use keyforge_protocol::{ResultSubmission, PROTOCOL_VERSION};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::oneshot;

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
pub async fn submit_result(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ResultSubmission>,
) -> AppResult<String> {
    if payload.version != PROTOCOL_VERSION {
        return Err(AppError::Validation("Protocol Mismatch".into()));
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    if now.abs_diff(payload.timestamp) > 900 {
        return Err(AppError::Validation("Submission expired".into()));
    }

    let nonce_key = format!("{}:{}", payload.node_id, payload.nonce);
    if state.nonce_cache.contains_key(&nonce_key) {
        return Err(AppError::Validation("Replay detected".into()));
    }
    state.nonce_cache.insert(nonce_key, true);

    // Delegate to Service
    state.verification.verify_submission(&payload).await?;

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

    if rx.await.is_err() {
        return Err(AppError::Any(anyhow::anyhow!("Persistence failed")));
    }

    Ok("Accepted".to_string())
}
