// apps/keyforge-hive/src/features/get_population.rs

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

use crate::error::AppResult;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    Json,
};
use keyforge_protocol::PopulationResponse;
use std::sync::Arc;

/// VSA Feature: Get Population
/// Returns the current population of results for a specific job.

#[utoipa::path(
    get,
    path = "/jobs/{job_id}/population",
    params(
        ("job_id" = String, Path, description = "Job ID")
    ),
    responses(
        (status = 200, description = "Current population", body = PopulationResponse)
    ),
    tag = "jobs"
)]
/// Handles a request to retrieve the current top layouts for a job.
#[tracing::instrument(skip_all)]
pub(crate) async fn handle(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> AppResult<Json<PopulationResponse>> {
    let layouts = state
        .results
        .get_population(&job_id)
        .await
        .map_err(crate::error::AppError::Database)?;

    Ok(Json(PopulationResponse { layouts }))
}
