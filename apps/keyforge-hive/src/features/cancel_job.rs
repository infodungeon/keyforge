// apps/keyforge-hive/src/features/cancel_job.rs

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

use axum::extract::{Path, State};
use std::sync::Arc;
use tracing::info;
use crate::error::AppResult;
use crate::state::AppState;

/// VSA Feature: Cancel Job
/// Terminates a job and notifies active workers.

#[utoipa::path(
    delete,
    path = "/jobs/{job_id}",
    params(
        ("job_id" = String, Path, description = "Job ID")
    ),
    responses(
        (status = 200, description = "Job cancelled")
    ),
    tag = "jobs"
)]
/// Handles a request to cancel an active job.
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> AppResult<String> {
    state
        .jobs
        .repo
        .cancel(&job_id)
        .await
        .map_err(crate::error::AppError::Database)?;

    let _ = state.tx.send(format!("CANCEL:{}", job_id));

    info!("🛑 Cancelled Job: {}", &job_id[0..8]);
    Ok("Job cancelled".to_string())
}
