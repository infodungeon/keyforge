use axum::{extract::State, Json};
use keyforge_protocol::{NodeRequest, NodeResponse, TuningProfile, Validator, PROTOCOL_VERSION};
use std::sync::Arc;
use tracing::info;
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

    // Stage 2: Persistence (Heartbeat)
    register_heartbeat(&state, &payload).await?;

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

fn validate_node_request(payload: &NodeRequest) -> AppResult<()> {
    // Flexible Version Check
    keyforge_protocol::check_version_compatibility(payload.version, PROTOCOL_VERSION)
        .map_err(AppError::Validation)?;

    // Public Key Format Check
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

async fn register_heartbeat(state: &AppState, payload: &NodeRequest) -> AppResult<()> {
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
        .map_err(|e| {
            if e.to_string().contains("Node Identity Mismatch") {
                AppError::Validation("Node Identity Mismatch".into())
            } else {
                AppError::Database(e)
            }
        })?;
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
