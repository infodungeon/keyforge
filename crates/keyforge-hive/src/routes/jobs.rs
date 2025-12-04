// ===== keyforge/crates/keyforge-hive/src/routes/jobs.rs =====
use axum::{
    extract::{Path, State},
    Json,
};
use keyforge_core::job::JobIdentifier;
use keyforge_core::protocol::RegisterJobRequest;
use serde::Serialize;
use std::sync::Arc;
use tokio::time::{sleep, Duration, Instant};
use tracing::info;

use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[derive(Serialize)]
pub struct RegisterJobResponse {
    pub job_id: String,
    pub is_new: bool,
}

#[derive(Serialize)]
pub struct JobQueueResponse {
    pub job_id: Option<String>,
    pub config: Option<RegisterJobRequest>,
}

#[derive(Serialize)]
pub struct PopulationResponse {
    pub layouts: Vec<String>,
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterJobRequest>,
) -> AppResult<Json<RegisterJobResponse>> {
    // Generate Deterministic ID based on ALL inputs
    let job_id = JobIdentifier::from_parts(
        &payload.definition.geometry, // Access geometry via definition
        &payload.weights,
        &payload.params,
        &payload.pinned_keys,
        &payload.corpus_name,
        &payload.cost_matrix,
    )
    .hash;

    // Check Existence
    if state.store.job_exists(&job_id).await {
        return Ok(Json(RegisterJobResponse {
            job_id,
            is_new: false,
        }));
    }

    // Register via Stored Procedure
    state
        .store
        .register_job(&job_id, &payload)
        .await
        .map_err(|e| AppError::Validation(format!("Registration failed: {}", e)))?;

    info!("🆕 Registered Job: {}", &job_id[0..8]);
    Ok(Json(RegisterJobResponse {
        job_id,
        is_new: true,
    }))
}

pub async fn get_queue(State(state): State<Arc<AppState>>) -> AppResult<Json<JobQueueResponse>> {
    let start = Instant::now();
    let timeout = Duration::from_secs(20);

    // Long polling loop
    loop {
        let result = state
            .store
            .get_latest_job()
            .await
            .map_err(|e| AppError::Any(anyhow::anyhow!(e)))?;

        if let Some((id, config)) = result {
            return Ok(Json(JobQueueResponse {
                job_id: Some(id),
                config: Some(config),
            }));
        }

        if start.elapsed() > timeout {
            return Ok(Json(JobQueueResponse {
                job_id: None,
                config: None,
            }));
        }

        sleep(Duration::from_secs(1)).await;
    }
}

pub async fn get_population(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> AppResult<Json<PopulationResponse>> {
    let layouts = state
        .store
        .get_job_population(&job_id)
        .await
        .map_err(|e| AppError::Any(anyhow::anyhow!(e)))?;

    Ok(Json(PopulationResponse { layouts }))
}
