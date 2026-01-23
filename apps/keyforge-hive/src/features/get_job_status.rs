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

use crate::constants::DEFAULT_STATUS_UNKNOWN;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    Json,
};
use keyforge_model::{Completed, JobStatus, Pending, Running, Score};
use keyforge_protocol::JobDetailedStatus;
use std::sync::Arc;

/// VSA Feature: Get Job Status
/// Returns comprehensive status and statistics for a job.

#[utoipa::path(
    get,
    path = "/jobs/{job_id}/status",
    params(
        ("job_id" = String, Path, description = "Job ID")
    ),
    responses(
        (status = 200, description = "Job status", body = JobDetailedStatus)
    ),
    tag = "jobs"
)]
/// Handles a request to retrieve the status and statistics for a job.
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> AppResult<Json<JobDetailedStatus>> {
    // 1. Fetch Status from Database
    let status_row = sqlx::query!("SELECT status FROM jobs WHERE id = $1", job_id)
        .fetch_optional(&state.jobs.repo.pool)
        .await
        .map_err(AppError::Database)?;

    let status_str = match status_row {
        Some(r) => r
            .status
            .unwrap_or_else(|| DEFAULT_STATUS_UNKNOWN.to_string()),
        None => return Err(AppError::NotFound),
    };

    // 2. Fetch Performance Stats
    let (nodes, samples, best_score, best_layout) = state
        .results
        .get_stats(&job_id)
        .await
        .map_err(AppError::Database)?;

    let status = match status_str.to_lowercase().as_str() {
        "running" => JobStatus::Running(Running {
            active_nodes: nodes,
            current_best: best_score
                .map(Score::from_f32)
                .transpose()
                .map_err(|e| AppError::Internal(format!("Invalid best score: {e}")))?
                .or(Some(Score::ZERO)),
        }),
        "completed" => JobStatus::Completed(Completed {
            final_score: best_score
                .map(Score::from_f32)
                .transpose()
                .map_err(|e| AppError::Internal(format!("Invalid final score: {e}")))?
                .unwrap_or(Score::ZERO),
            final_layout: best_layout
                .clone()
                .and_then(|l| serde_json::from_str(&l).ok())
                .unwrap_or_else(|| keyforge_model::Layout::new_unchecked(vec![])),
            total_compute_sec: 0, // TODO: Aggregate from DB
        }),
        _ => JobStatus::Pending(Pending),
    };

    Ok(Json(JobDetailedStatus {
        job_id,
        status,
        best_score,
        best_layout,
        total_samples: samples,
    }))
}
