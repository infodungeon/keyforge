// apps/keyforge-hive/src/features/register_node.rs

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
use keyforge_model::Validator;
use keyforge_protocol::{NodeRequest, NodeResponse, TuningProfile, PROTOCOL_VERSION};
use std::sync::Arc;
use tracing::{info, warn, debug};
use crate::error::{AppError, AppResult};
use crate::state::AppState;

/// VSA Feature: Register Node
/// Handles node heartbeat, identity verification, and auto-tuning calculations.
#[utoipa::path(
    post,
    path = "/nodes/register",
    request_body = NodeRequest,
    responses(
        (status = 200, description = "Node registered", body = NodeResponse)
    ),
    tag = "nodes"
)]
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<NodeRequest>,
) -> AppResult<Json<NodeResponse>> {
    // Stage 1: Validation
    payload.validate().map_err(AppError::Validation)?;
    validate_node_request(&payload)?;

    // Stage 2: Persistence (Optimized)
    // We use Valkey to check if the Hardware Profile is already known.
    // If Known: Use "Lite" insert (Nodes table only) -> No contention.
    // If Unknown: Use "Full" insert (Hardware Profiles + Nodes) -> Contention possible but rare.
    
    let is_new_profile = state.coordinator
        .try_reserve_profile_update(&payload.cpu_model)
        .await
        .unwrap_or(true); // Fail-safe: Assume new if Valkey down

    if is_new_profile {
        // FULL PATH: Updates Hardware Profiles + Nodes
        debug!("📝 Registering NEW Hardware Profile: {}", payload.cpu_model);
        state.nodes.register_heartbeat(
            &payload.node_id,
            &payload.cpu_model,
            payload.cores,
            payload.l2_cache_kb,
            payload.ops_per_sec,
            payload.public_key.as_deref(),
        ).await.map_err(|e| map_db_error(e))?;
    } else {
        // LITE PATH: Updates Nodes Only (Optimistic)
        // If this fails (e.g. FK violation because Valkey was wrong), fallback to Full.
        if let Err(e) = state.nodes.register_heartbeat_lite(
            &payload.node_id,
            &payload.cpu_model,
            payload.cores,
            payload.ops_per_sec,
            payload.public_key.as_deref()
        ).await {
            warn!("⚠️ Lite registration failed (Fallback to Full): {}", e);
            
            // FALLBACK
            state.nodes.register_heartbeat(
                &payload.node_id,
                &payload.cpu_model,
                payload.cores,
                payload.l2_cache_kb,
                payload.ops_per_sec,
                payload.public_key.as_deref(),
            ).await.map_err(|e| map_db_error(e))?;
        }
    }

    // Stage 3: Auto-Tuning
    let tuning = calculate_tuning_profile(&payload);

    info!(
        "🖥️ Node Registered: {} | {} | {:.1} M/s",
        payload.node_id,
        payload.cpu_model,
        payload.ops_per_sec / 1_000_000.0
    );

    Ok(Json(NodeResponse {
        status: "registered".to_string(),
        tuning,
    }))
}

fn map_db_error(e: sqlx::Error) -> AppError {
    if e.to_string().contains("Node Identity Mismatch") {
        AppError::Validation("Node Identity Mismatch".into())
    } else {
        AppError::Database(e)
    }
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
        if l2 >= 1024 { "table" } else { "fly" }
    } else {
        "fly"
    };

    let batch_size = if payload.ops_per_sec > 10_000_000.0 {
        50_000
    } else {
        10_000
    };
    let thread_count = (payload.cores - 1).max(1) as usize;

    TuningProfile {
        strategy: strategy.to_string(),
        batch_size,
        thread_count,
    }
}
