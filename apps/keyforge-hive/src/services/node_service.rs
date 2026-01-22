// apps/keyforge-hive/src/services/node_service.rs

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

//! Service for orchestrating node registration and lifecycle.

use crate::constants::{
    TUNING_BATCH_SIZE_LARGE, TUNING_BATCH_SIZE_SMALL, TUNING_L2_CACHE_THRESHOLD,
    TUNING_OPS_THRESHOLD,
};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use keyforge_protocol::{NodeRequest, NodeResponse, TuningProfile, PROTOCOL_VERSION};
use keyforge_security as crypto;
use tracing::{debug, warn};

pub struct NodeService;

impl NodeService {
    /// Registers a new node and issues a session token.
    pub async fn register_node(state: &AppState, payload: NodeRequest) -> AppResult<NodeResponse> {
        // 1. Validation
        Self::validate_node_request(&payload)?;

        // 2. Persistence (Hardware Profile Management)
        let is_new_profile = state
            .coordinator
            .try_reserve_profile_update(&payload.cpu_model)
            .await
            .unwrap_or(true);

        if is_new_profile {
            debug!("📝 Registering NEW Hardware Profile: {}", payload.cpu_model);
            state
                .nodes
                .register_heartbeat(
                    &payload.node_id,
                    &payload.cpu_model,
                    payload.cores,
                    payload.l2_cache_kb,
                    payload.ops_per_sec,
                    payload.public_key.as_deref(),
                )
                .await
                .map_err(Self::map_db_error)?;
        } else {
            if let Err(e) = state
                .nodes
                .register_heartbeat_lite(
                    &payload.node_id,
                    &payload.cpu_model,
                    payload.cores,
                    payload.ops_per_sec,
                    payload.public_key.as_deref(),
                )
                .await
            {
                warn!("⚠️ Lite registration failed (Fallback to Full): {}", e);
                state
                    .nodes
                    .register_heartbeat(
                        &payload.node_id,
                        &payload.cpu_model,
                        payload.cores,
                        payload.l2_cache_kb,
                        payload.ops_per_sec,
                        payload.public_key.as_deref(),
                    )
                    .await
                    .map_err(Self::map_db_error)?;
            }
        }

        // 3. Auto-Tuning
        let tuning = Self::calculate_tuning_profile(&payload);

        // 4. Token Issuance
        let token_key = state.security.get_token_key();
        let token = crypto::create_paseto_token(&token_key, &payload.node_id, 86400 * 7)
            .map_err(|e| AppError::Internal(format!("Token issuance failed: {e}")))?;

        Ok(NodeResponse {
            status: "registered".to_string(),
            tuning,
            token: Some(token),
        })
    }

    fn validate_node_request(payload: &NodeRequest) -> AppResult<()> {
        keyforge_protocol::check_version_compatibility(payload.version, PROTOCOL_VERSION)
            .map_err(AppError::Validation)?;

        if let Some(pk) = &payload.public_key {
            if pk.len() < 64
                || (!pk.starts_with("-----BEGIN PUBLIC KEY")
                    && !pk.chars().all(|c| c.is_ascii_hexdigit()))
            {
                return Err(AppError::Validation(
                    "Invalid Public Key Format (PEM or Hex required)".into(),
                ));
            }
        }
        Ok(())
    }

    fn calculate_tuning_profile(payload: &NodeRequest) -> TuningProfile {
        let strategy = if let Some(l2) = payload.l2_cache_kb {
            #[allow(clippy::cast_possible_wrap)]
            if l2 >= TUNING_L2_CACHE_THRESHOLD as i32 {
                "table"
            } else {
                "fly"
            }
        } else {
            "fly"
        };

        let batch_size = if payload.ops_per_sec > TUNING_OPS_THRESHOLD {
            TUNING_BATCH_SIZE_LARGE
        } else {
            TUNING_BATCH_SIZE_SMALL
        };
        #[allow(clippy::cast_sign_loss)]
        let thread_count = (payload.cores - 1).max(1) as usize;

        TuningProfile {
            strategy: strategy.to_string(),
            batch_size,
            thread_count,
        }
    }

    fn map_db_error(e: sqlx::Error) -> AppError {
        if e.to_string().contains("Node Identity Mismatch") {
            AppError::Validation("Node Identity Mismatch".into())
        } else {
            AppError::Database(e)
        }
    }
}
