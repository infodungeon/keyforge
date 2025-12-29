use axum::{extract::State, Json};
use keyforge_protocol::JobConfig;
use keyforge_protocol::JobQueueResponse;
use std::sync::Arc;
use tokio::time::{sleep, Duration, Instant};
use crate::error::{AppError, AppResult};
use crate::state::AppState;

/// VSA Feature: Get Job Queue (Long Polling)
/// Colocates claiming logic, wait signals, and semaphore management.

#[utoipa::path(
    get,
    path = "/jobs/queue",
    responses(
        (status = 200, description = "Next available job", body = JobQueueResponse)
    ),
    tag = "jobs"
)]
pub async fn handle(State(state): State<Arc<AppState>>) -> AppResult<Json<JobQueueResponse>> {
    // 1. Limit Concurrency via Semaphore (HIVE-006)
    let _permit = state
        .poll_semaphore
        .acquire()
        .await
        .map_err(|_| AppError::Any(anyhow::anyhow!("Semaphore closed")))?;

    let result = poll_for_job(&state).await?;
    
    Ok(Json(result))
}

async fn poll_for_job(state: &AppState) -> AppResult<JobQueueResponse> {
    let start = Instant::now();
    let timeout = Duration::from_secs(20);

    loop {
        // 1. Attempt to claim a job from the database
        if let Some((id, req)) = state.jobs.claim_job().await.map_err(AppError::Database)? {
            return Ok(JobQueueResponse {
                job_id: Some(id),
                config: Some(JobConfig::from(req)),
            });
        }

        // 2. Check if we've exceeded our long-poll timeout
        if start.elapsed() > timeout {
            return Ok(JobQueueResponse {
                job_id: None,
                config: None,
            });
        }

        // 3. Wait for New Job Signal or Remaining Timeout
        let remaining = timeout.saturating_sub(start.elapsed());

        tokio::select! {
            _ = state.job_signal.notified() => {
                // Return to start of loop to re-check DB
                continue;
            }
            _ = sleep(remaining) => {
                return Ok(JobQueueResponse {
                    job_id: None,
                    config: None,
                });
            }
        }
    }
}
