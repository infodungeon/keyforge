// apps/keyforge-hive/src/services/job_service.rs

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use keyforge_compute::use_cases::OptimizationUseCase;
use keyforge_protocol::{CostMatrixSourceDto, JobRequest, JobResponse};
use tracing::info;

pub struct JobService;

impl JobService {
    pub async fn register_job(state: &AppState, mut payload: JobRequest) -> AppResult<JobResponse> {
        // 0. Sanity check
        Self::validate_config(&payload)?;

        // 1. Convert DTO to Domain for ID generation using Unified Use Case
        // In Hive, we only need the ID for registration, but the Use Case ensures
        // consistency with the CLI.
        let (id, session) = OptimizationUseCase::prepare_session(state.assets.as_ref(), &payload)
            .await
            .map_err(|e| AppError::Validation(e.to_string()))?;

        // 2. Enrich the payload with the full geometry from the loader if it was missing
        // This ensures the database gets the full definition even if the client sent a sparse one.
        payload.config.definition = (*session.keyboard).clone().into();

        let job_id = id.hash;
        info!("📝 Registering Job: {}", job_id);

        let is_new = state
            .jobs
            .repo
            .register(
                &job_id,
                &payload,
                None,
                payload.config.parent_job_id.clone(),
                0,
            )
            .await?;

        if is_new {
            let _ = state.coordinator.publish_update(&job_id, "new_job").await;
        }

        Ok(JobResponse { job_id, is_new })
    }

    pub fn validate_config(req: &JobRequest) -> AppResult<()> {
        match &req.config.cost_matrix {
            CostMatrixSourceDto::Predefined(name) => {
                if name.trim().is_empty() {
                    return Err(AppError::Validation(
                        "Cost matrix name cannot be empty".into(),
                    ));
                }
            }
        }
        Ok(())
    }
}
