use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use keyforge_core::{config::ScoringWeights, geometry::KeyboardGeometry, job::JobIdentifier};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::time::{sleep, Duration, Instant};
use tracing::info;

use crate::state::AppState;

#[derive(Deserialize, Serialize, Clone)]
pub struct RegisterJobRequest {
    pub geometry: KeyboardGeometry,
    pub weights: ScoringWeights,
    pub pinned_keys: String,
    pub corpus_name: String,
}

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
) -> Result<Json<RegisterJobResponse>, (StatusCode, String)> {
    let job_id = JobIdentifier::from_parts(
        &payload.geometry,
        &payload.weights,
        &payload.pinned_keys,
        &payload.corpus_name,
    )
    .hash;

    if state.store.job_exists(&job_id).await {
        return Ok(Json(RegisterJobResponse {
            job_id,
            is_new: false,
        }));
    }

    state
        .store
        .register_job(&job_id, &payload)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    info!("🆕 Registered Job: {}", &job_id[0..8]);
    Ok(Json(RegisterJobResponse {
        job_id,
        is_new: true,
    }))
}

pub async fn get_queue(
    State(state): State<Arc<AppState>>,
) -> Result<Json<JobQueueResponse>, (StatusCode, String)> {
    // LONG POLLING:
    // Attempt to find a job for up to 20 seconds.
    // This allows the worker node to hold the connection open rather than spamming requests.
    let start = Instant::now();
    let timeout = Duration::from_secs(20);

    loop {
        let result = state
            .store
            .get_latest_job()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

        if let Some((id, config)) = result {
            return Ok(Json(JobQueueResponse {
                job_id: Some(id),
                config: Some(config),
            }));
        }

        if start.elapsed() > timeout {
            // Timeout reached, tell client to retry later
            return Ok(Json(JobQueueResponse {
                job_id: None,
                config: None,
            }));
        }

        // Wait 1s before checking DB again to prevent tight loops
        sleep(Duration::from_secs(1)).await;
    }
}

pub async fn get_population(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> Result<Json<PopulationResponse>, (StatusCode, String)> {
    let layouts = state
        .store
        .get_job_population(&job_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(PopulationResponse { layouts }))
}
