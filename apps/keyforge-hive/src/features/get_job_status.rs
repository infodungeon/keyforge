// apps/keyforge-hive/src/features/get_job_status.rs

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

use axum::{extract::{Path, State}, Json};
use keyforge_protocol::JobStatus;
use std::sync::Arc;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

/// VSA Feature: Get Job Status
/// Returns comprehensive status and statistics for a job.

#[utoipa::path(
    get,
    path = "/jobs/{job_id}/status",
    params(
        ("job_id" = String, Path, description = "Job ID")
    ),
    responses(
        (status = 200, description = "Job status", body = JobStatus)
    ),
    tag = "jobs"
)]
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> AppResult<Json<JobStatus>> {
    // 1. Fetch Status from Database
    let status_row = sqlx::query!("SELECT status FROM jobs WHERE id = $1", job_id)
        .fetch_optional(&state.jobs.repo.pool)
        .await
        .map_err(AppError::Database)?;

    let status = match status_row {
        Some(r) => r.status.unwrap_or_else(|| "unknown".to_string()),
        None => return Err(AppError::NotFound),
    };

    // 2. Fetch Performance Stats
    let (nodes, samples, best_score, best_layout) = state
        .results
        .get_stats(&job_id)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(JobStatus {
        job_id,
        status,
        active_nodes: nodes,
        best_score,
        best_layout,
        total_samples: samples,
    }))
}
