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
use keyforge_protocol::types::ScoreDto;
use keyforge_protocol::{JobDetailedStatus, JobStatusDto, LayoutDto};
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
#[tracing::instrument(skip_all)]
pub(crate) async fn handle(
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
        "running" | "processing" => JobStatusDto::Running {
            active_nodes: usize::try_from(nodes).unwrap_or(0),
            current_best: best_score.map(|s| {
                ScoreDto(
                    keyforge_model::types::Score::from_f32(s)
                        .unwrap_or_default().raw(),
                )
            }),
        },
        "completed" => {
            let score_val = best_score.map_or(0, |s| {
                keyforge_model::types::Score::from_f32(s)
                    .unwrap_or_default().raw()
            });
            let layout = best_layout
                .clone()
                .and_then(|l| serde_json::from_str::<keyforge_model::Layout>(&l).ok())
                .map(LayoutDto::from)
                .unwrap_or_default();

            JobStatusDto::Completed {
                final_score: ScoreDto(score_val),
                final_layout: layout,
                // ESTIMATE: Assume average 50ms per sample until schema tracks duration
                total_compute_sec: (u64::try_from(samples).unwrap_or(0) * 50) / 1000,
            }
        }
        _ => JobStatusDto::Pending,
    };

    Ok(Json(JobDetailedStatus {
        job_id,
        status,
        best_score,
        best_layout,
        total_samples: usize::try_from(samples).unwrap_or(0),
    }))
}
