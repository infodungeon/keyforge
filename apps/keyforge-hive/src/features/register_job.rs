use axum::{extract::State, Json};
use keyforge_protocol::job::JobIdentifier;
use keyforge_protocol::{CostMatrixSource, JobRequest, JobResponse, Validator, PROTOCOL_VERSION};
use std::sync::Arc;
use tracing::{info, warn};

use crate::error::{AppError, AppResult};
use crate::state::AppState;

/// VSA Feature: Register Job
/// Colocates API logic, validation, and process flow.

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
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<JobRequest>,
) -> AppResult<Json<JobResponse>> {
    // Humble Handler: Plumb Axum state to the domain process
    let result = process_job_registration(&state, payload).await?;
    
    Ok(Json(result))
}

/// The core domain process, designed using Railway Oriented Programming (ROP).
/// This function is decoupled from Axum extracts, making it easily testable.
async fn process_job_registration(state: &AppState, mut payload: JobRequest) -> AppResult<JobResponse> {
    // Stage 1: Validation
    validate_request(&payload)?;

    // Stage 2: Resolution (IO/State dependent)
    resolve_assets(state, &mut payload).await?;

    // Stage 3: Identification
    let job_id = generate_job_id(&payload)?;

    // Stage 4: Persistence
    let is_new = state
        .jobs
        .repo
        .register(&job_id, &payload, None, payload.parent_job_id.clone(), 0)
        .await
        .map_err(AppError::Database)?;

    // Stage 5: Side Effects
    if is_new {
        emit_registration_events(state, &job_id);
    }

    Ok(JobResponse { job_id, is_new })
}

fn validate_request(payload: &JobRequest) -> AppResult<()> {
    if payload.version != PROTOCOL_VERSION {
        return Err(AppError::Validation(format!(
            "Protocol Mismatch. Server: v{}, Client: v{}",
            PROTOCOL_VERSION, payload.version
        )));
    }

    payload
        .validate()
        .map_err(|e| AppError::Validation(format!("Invalid Job Request: {}", e)))?;

    validate_input_safety(payload)?;
    Ok(())
}

async fn resolve_assets(state: &AppState, payload: &mut JobRequest) -> AppResult<()> {
    for corpus in &mut payload.corpora {
        if corpus.hash.is_none() {
            let hash = state
                .assets
                .get_corpus_hash(&corpus.id)
                .await
                .map_err(|e| AppError::Validation(format!("Corpus error: {}", e)))?;
            corpus.hash = Some(hash);
        }
    }
    Ok(())
}

fn generate_job_id(payload: &JobRequest) -> AppResult<String> {
    let corpora_fingerprint =
        serde_json::to_string(&payload.corpora).unwrap_or_else(|_| "default".to_string());

    let id = JobIdentifier::try_from_parts(
        &payload.definition.geometry,
        &payload.weights,
        &payload.params,
        &payload.pinned_keys,
        &corpora_fingerprint,
        &payload.cost_matrix,
    )
    .map_err(|e| AppError::Validation(format!("job id generation failed: {}", e)))?;
    
    Ok(id.hash)
}

fn emit_registration_events(state: &AppState, job_id: &str) {
    let _ = state.tx.send(format!("JOB:{}", job_id));
    state.jobs.signal.notify_waiters();
    info!("🆕 (VSA/Humble/ROP) Registered Job: {}", &job_id[0..8]);
}

/// Internal safety validation specific to Hive's filesystem/path assumptions.
fn validate_input_safety(req: &JobRequest) -> AppResult<()> {
    match &req.cost_matrix {
        CostMatrixSource::Predefined(name) => {
            crate::api::validation::validate_filename(name).map_err(|e| {
                warn!("Security Alert: Invalid cost_matrix path: {} ({})", name, e);
                e
            })?;
        }
        CostMatrixSource::Custom(_) => {}
    }

    for c in &req.corpora {
        crate::api::validation::validate_path_component(&c.id).map_err(|e| {
            warn!("Security Alert: Invalid corpus ID: {} ({})", c.id, e);
            e
        })?;
    }
    Ok(())
}
