// apps/keyforge-hive/src/features/get_queue.rs

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
use keyforge_protocol::JobConfig;
use keyforge_protocol::JobQueueResponse;
use std::sync::Arc;
use tokio::time::{sleep, Duration, Instant};

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
/// Handles a long-polling request to claim the next available job from the queue.
#[tracing::instrument(skip_all)] pub(crate) async fn handle(State(state): State<Arc<AppState>>) -> AppResult<Json<JobQueueResponse>> {
    // 1. Limit Concurrency via Semaphore (HIVE-006)
    let _permit = state
        .jobs
        .semaphore
        .acquire()
        .await
        .map_err(|_| AppError::Any(anyhow::anyhow!("Semaphore closed")))?;

    let result = poll_for_job(&state).await?;

    Ok(Json(result))
}

/// Polls the database for an available job, waiting for a signal if none are found.
async fn poll_for_job(state: &AppState) -> AppResult<JobQueueResponse> {
    let start = Instant::now();
    let timeout = Duration::from_secs(state.config.network.timeout_seconds);

    loop {
        // 1. Attempt to claim a job from the database
        if let Some((id, req)) = state
            .jobs
            .repo
            .claim_job()
            .await
            .map_err(AppError::Database)?
        {
            state
                .jobs
                .active_count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
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
            () = state.jobs.signal.notified() => {
                // Return to start of loop to re-check DB
            }
            () = sleep(remaining) => {
                return Ok(JobQueueResponse {
                    job_id: None,
                    config: None,
                });
            }
        }
    }
}
