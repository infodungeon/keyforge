// apps/keyforge-hive/src/services/job_service.rs

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

//! Central service for orchestrating job-related operations.

use keyforge_model::{CostMatrixSource, JobIdentifier, Validator};
use keyforge_protocol::{JobRequest, JobResponse, PROTOCOL_VERSION};
use tracing::{info, warn};

use crate::constants::{DEFAULT_JOB_PRIORITY, LOG_JOB_ID_TRUNCATION};
use crate::error::{AppError, AppResult};
use crate::state::AppState;

/// Service for handling high-level job orchestration.
#[derive(Debug)]
pub struct JobService;

impl JobService {
    /// Registers a new job by validating the request, resolving assets, and persisting to the database.
    pub async fn register_job(state: &AppState, mut payload: JobRequest) -> AppResult<JobResponse> {
        // 1. Semantic Validation
        Self::validate_request(&payload)?;
        
        // 2. Security/Boundary Validation
        Self::validate_input_safety(&payload)?;

        // 3. Asset Resolution (Hydrate missing hashes)
        Self::resolve_assets(state, &mut payload).await?;

        // 4. Deterministic ID Generation
        let job_id = Self::generate_job_id(&payload)?;

        // 5. Database Transaction/Persistence
        let is_new = state
            .jobs
            .repo
            .register(
                &job_id,
                &payload,
                None,
                payload.config.parent_job_id.clone(),
                DEFAULT_JOB_PRIORITY,
            )
            .await
            .map_err(AppError::Database)?;

        // 6. Side Effects (Events/Logging)
        if is_new {
            let _ = state.tx.send(format!("JOB:{job_id}"));
            state.jobs.signal.notify_waiters();
            info!(
                "🆕 (JobService) Registered Job: {}",
                &job_id[0..LOG_JOB_ID_TRUNCATION]
            );
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
            .map_err(|e| AppError::Validation(format!("Invalid Job Request: {e}")))?;

        Ok(())
    }

    async fn resolve_assets(state: &AppState, payload: &mut JobRequest) -> AppResult<()> {
        for corpus in &mut payload.config.corpora {
            if corpus.hash.is_none() {
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

    fn generate_job_id(payload: &JobRequest) -> AppResult<String> {
        let corpora_fingerprint =
            keyforge_infra::util::common::calculate_fingerprint(&payload.config.corpora);

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
}
