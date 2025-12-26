use axum::{
    extract::{Path, State},
    Json,
};
use keyforge_protocol::job::JobIdentifier;
use keyforge_protocol::{
    CostMatrixSource, JobConfig, JobQueueResponse, JobRequest, JobResponse, JobStatus,
    PopulationResponse, Validator, PROTOCOL_VERSION,
};
use std::sync::Arc;
use tokio::time::{sleep, Duration, Instant};
use tracing::{info, warn};

use crate::error::{AppError, AppResult};
use crate::state::AppState;

// SECURITY: Input Sanitization (SEC-005)
fn validate_input_safety(req: &JobRequest) -> AppResult<()> {
    match &req.cost_matrix {
        CostMatrixSource::Predefined(name) => {
            crate::api::validation::validate_filename(name).map_err(|e| {
                warn!("Security Alert: Invalid cost_matrix path: {} ({})", name, e);
                e
            })?;
        }
        CostMatrixSource::Custom(_) => {
            // Content validation is handled in Validator trait
        }
    }

    for c in &req.corpora {
        crate::api::validation::validate_path_component(&c.id).map_err(|e| {
            warn!("Security Alert: Invalid corpus ID: {} ({})", c.id, e);
            e
        })?;
    }
    Ok(())
}

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
pub async fn register_job(
    State(state): State<Arc<AppState>>,
    Json(mut payload): Json<JobRequest>,
) -> AppResult<Json<JobResponse>> {
    if payload.version != PROTOCOL_VERSION {
        return Err(AppError::Validation(format!(
            "Protocol Mismatch. Server: v{}, Client: v{}",
            PROTOCOL_VERSION, payload.version
        )));
    }

    payload
        .validate()
        .map_err(|e| AppError::Validation(format!("Invalid Job Request: {}", e)))?;
    validate_input_safety(&payload)?;

    for corpus in &mut payload.corpora {
        if corpus.hash.is_none() {
            let hash = state
                .assets
                .get_corpus_hash(&corpus.id)
                .map_err(|e| AppError::Validation(format!("Corpus error: {}", e)))?;
            corpus.hash = Some(hash);
        }
    }

    let corpora_fingerprint =
        serde_json::to_string(&payload.corpora).unwrap_or_else(|_| "default".to_string());

    let job_id = JobIdentifier::try_from_parts(
        &payload.definition.geometry,
        &payload.weights,
        &payload.params,
        &payload.pinned_keys,
        &corpora_fingerprint,
        &payload.cost_matrix,
    )
    .map_err(|e| AppError::Validation(format!("job id generation failed: {}", e)))?
    .hash;

    let is_new = state
        .jobs
        .register(&job_id, &payload, None, payload.parent_job_id.clone(), 0)
        .await
        .map_err(AppError::Database)?;

    if is_new {
        let _ = state.tx.send(format!("JOB:{}", job_id));
        // Wake up polling workers
        state.job_signal.notify_waiters();
        info!("🆕 Registered Job: {}", &job_id[0..8]);
    }

    Ok(Json(JobResponse { job_id, is_new }))
}

#[utoipa::path(
    get,
    path = "/jobs/queue",
    responses(
        (status = 200, description = "Next available job", body = JobQueueResponse)
    ),
    tag = "jobs"
)]
pub async fn get_queue(State(state): State<Arc<AppState>>) -> AppResult<Json<JobQueueResponse>> {
    // HIVE-006: Limit Concurrency via Semaphore
    let _permit = state
        .poll_semaphore
        .acquire()
        .await
        .map_err(|_| AppError::Any(anyhow::anyhow!("Semaphore closed")))?;

    let start = Instant::now();
    let timeout = Duration::from_secs(20);

    loop {
        // 1. Check DB
        let result = state.jobs.claim_job().await.map_err(AppError::Database)?;

        if let Some((id, req)) = result {
            let config = JobConfig::from(req);
            return Ok(Json(JobQueueResponse {
                job_id: Some(id),
                config: Some(config),
            }));
        }

        // 2. Check Timeout
        if start.elapsed() > timeout {
            return Ok(Json(JobQueueResponse {
                job_id: None,
                config: None,
            }));
        }

        // 3. Wait for Signal (Long Poll)
        // We wait for either a notification (new job) or the remaining timeout
        let remaining = timeout.saturating_sub(start.elapsed());

        tokio::select! {
            _ = state.job_signal.notified() => {
                // New job arrived, loop back to check DB
                continue;
            }
            _ = sleep(remaining) => {
                // Timeout reached
                return Ok(Json(JobQueueResponse {
                    job_id: None,
                    config: None,
                }));
            }
        }
    }
}

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
pub async fn get_population(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> AppResult<Json<PopulationResponse>> {
    let layouts = state
        .results
        .get_population(&job_id)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(PopulationResponse { layouts }))
}

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
pub async fn cancel_job(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> AppResult<String> {
    state
        .jobs
        .cancel(&job_id)
        .await
        .map_err(AppError::Database)?;

    let _ = state.tx.send(format!("CANCEL:{}", job_id));

    info!("🛑 Cancelled Job: {}", &job_id[0..8]);
    Ok("Job cancelled".to_string())
}

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
pub async fn get_job_status(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> AppResult<Json<JobStatus>> {
    // Check if job exists and get status
    let status_row = sqlx::query!("SELECT status FROM jobs WHERE id = $1", job_id)
        .fetch_optional(&state.jobs.pool)
        .await
        .map_err(AppError::Database)?;

    let status = match status_row {
        Some(r) => r.status.unwrap_or_else(|| "unknown".to_string()),
        None => return Err(AppError::NotFound),
    };

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
