// apps/keyforge-hive/src/services/result_service.rs

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

//! Service for orchestrating result submission and verification.

use crate::config::DEFAULT_SUBMISSION_EXPIRATION_SECS;
use crate::error::{AppError, AppResult};
use crate::infra::queue::PersistedRecord;
use crate::state::AppState;
use keyforge_model::Validator;
use keyforge_protocol::{ResultSubmission, PROTOCOL_VERSION};

/// Service for handling result submissions.
pub struct ResultService;

impl ResultService {
    /// Submits a verified result to the system.
    pub async fn submit_result(state: &AppState, payload: ResultSubmission) -> AppResult<bool> {
        Self::validate_submission(state, &payload).await?;
        Self::persist_result(state, payload)?;
        Ok(true)
    }

    async fn validate_submission(state: &AppState, payload: &ResultSubmission) -> AppResult<()> {
        if payload.version != PROTOCOL_VERSION {
            return Err(AppError::Validation(format!(
                "Protocol Mismatch. Server: v{PROTOCOL_VERSION}, Client: v{}",
                payload.version
            )));
        }

        payload.validate().map_err(AppError::Validation)?;
        state.verification.verify_submission(payload).await?;

        // Replay Protection
        #[allow(clippy::cast_possible_wrap)]
        let is_new = state
            .coordinator
            .check_and_set_nonce(
                &payload.node_id,
                payload.nonce,
                DEFAULT_SUBMISSION_EXPIRATION_SECS as i64,
            )
            .await
            .map_err(|e| AppError::Any(anyhow::anyhow!("Valkey Error: {e}")))?;

        if !is_new {
            return Err(AppError::Validation("Replay detected".into()));
        }

        Ok(())
    }

    fn persist_result(state: &AppState, payload: ResultSubmission) -> AppResult<()> {
        state
            .queue
            .push(PersistedRecord {
                job_id: payload.job_id,
                layout: payload.layout,
                score: payload.score,
                node_id: payload.node_id,
            })
            .map_err(|e| {
                if e == "Queue full" {
                    AppError::ServiceUnavailable("Persistence queue full".into())
                } else {
                    AppError::Any(anyhow::anyhow!("Persistence failed: {e}"))
                }
            })?;

        Ok(())
    }
}
