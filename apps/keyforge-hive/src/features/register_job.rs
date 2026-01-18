// apps/keyforge-hive/src/features/register_job.rs

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
use keyforge_model::{CostMatrixSource, JobIdentifier, Validator};
use keyforge_protocol::{JobRequest, JobResponse, PROTOCOL_VERSION};
use std::sync::Arc;
use tracing::{info, warn};

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::constants::{DEFAULT_JOB_PRIORITY, LOG_JOB_ID_TRUNCATION};

#[utoipa::path(
    post,
    path = "/jobs",
    request_body = JobRequest,
    responses(
        (status = 200, description = "Job registered successfully", body = JobResponse),
        (status = 400, description = "Invalid request parameters")
    ),
    tag = "jobs"
)]
/// Handles a request to register a new optimization job.
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<JobRequest>,
) -> AppResult<Json<JobResponse>> {
    let result = process_job_registration(&state, payload).await?;
    Ok(Json(result))
}

/// Orchestrates the job registration flow: validation, asset resolution, and database persistence.
async fn process_job_registration(state: &AppState, mut payload: JobRequest) -> AppResult<JobResponse> {
    validate_request(&payload)?;
    resolve_assets(state, &mut payload).await?;
    let job_id = generate_job_id(&payload)?;

    let is_new = state
        .jobs
        .repo
        .register(&job_id, &payload, None, payload.config.parent_job_id.clone(), DEFAULT_JOB_PRIORITY)
        .await
        .map_err(AppError::Database)?;

    if is_new {
        emit_registration_events(state, &job_id);
    }

    Ok(JobResponse { job_id, is_new })
}

/// Validates the protocol version and request parameters.
fn validate_request(payload: &JobRequest) -> AppResult<()> {
    if payload.version != PROTOCOL_VERSION {
        return Err(AppError::Validation(format!(
            "Protocol Mismatch. Server: v{}, Client: v{}",
            PROTOCOL_VERSION, payload.version
        )));
    }

    payload
        .validate()
        .map_err(|e| AppError::Validation(format!("Invalid Job Request: {e}")))?;

    validate_input_safety(payload)?;
    Ok(())
}

/// Resolves corpus hashes if they were not provided in the request.
async fn resolve_assets(state: &AppState, payload: &mut JobRequest) -> AppResult<()> {
    for corpus in &mut payload.config.corpora {
        if corpus.hash.is_none() {
            // FIX: Await the async hash retrieval
            let hash = state
                .assets
                .get_corpus_hash(&corpus.id)
                .await
                .map_err(|e| AppError::Validation(format!("Corpus error: {e}")))?;
            corpus.hash = Some(hash);
        }
    }
    Ok(())
}

/// Generates a deterministic job ID based on the job configuration.
fn generate_job_id(payload: &JobRequest) -> AppResult<String> {
    let corpora_fingerprint = keyforge_infra::util::common::calculate_fingerprint(&payload.config.corpora);

    let id = JobIdentifier::try_from_parts(
        &payload.config.definition.geometry,
        &payload.config.weights,
        &payload.config.params,
        &payload.config.pinned_keys,
        &corpora_fingerprint,
        &payload.config.cost_matrix,
    )
    .map_err(|e| AppError::Validation(format!("job id generation failed: {e}")))?;
    
    Ok(id.hash)
}

/// Notifies waiters and logs the registration of a new job.
fn emit_registration_events(state: &AppState, job_id: &str) {
    let _ = state.tx.send(format!("JOB:{job_id}"));
    state.jobs.signal.notify_waiters();
    info!("🆕 (VSA/Humble/ROP) Registered Job: {}", &job_id[0..LOG_JOB_ID_TRUNCATION]);
}

/// Performs security-related validation on input paths and IDs.
fn validate_input_safety(req: &JobRequest) -> AppResult<()> {
    match &req.config.cost_matrix {
        CostMatrixSource::Predefined(name) => {
            crate::api::validation::validate_filename(name).map_err(|e| {
                warn!("Security Alert: Invalid cost_matrix path: {} ({})", name, e);
                e
            })?;
        }
    }

    for c in &req.config.corpora {
        crate::api::validation::validate_path_component(&c.id).map_err(|e| {
            warn!("Security Alert: Invalid corpus ID: {} ({})", c.id, e);
            e
        })?;
    }
    Ok(())
}
