// apps/keyforge-hive/src/services/job_service.rs

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use keyforge_model::job::JobIdentifier;
use keyforge_protocol::{CostMatrixSourceDto, JobRequest, JobResponse};
use tracing::info;

pub struct JobService;

impl JobService {
    pub async fn register_job(state: &AppState, payload: JobRequest) -> AppResult<JobResponse> {
        // 0. Sanity check
        Self::validate_config(&payload)?;

        // 1. Convert DTO to Domain for ID generation
        let geometry = payload.config.to_domain_geometry();
        let weights = payload.config.to_domain_weights();
        let params = payload.config.to_domain_params();
        let pinned = payload.config.to_domain_pinned_keys();
        let corpora = payload.config.to_domain_corpus_sources();
        let cost_matrix = payload.config.to_domain_cost_matrix();

        let corpora_fingerprint = keyforge_infra::util::common::calculate_fingerprint(&corpora);

        let id = JobIdentifier::try_from_parts(
            &geometry,
            &weights,
            &params,
            &pinned,
            &corpora_fingerprint,
            &cost_matrix,
        )
        .map_err(|e| AppError::Validation(e.to_string()))?;

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
