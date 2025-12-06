use axum::{extract::State, Json};
use keyforge_protocol::protocol::{RegisterNodeRequest, RegisterNodeResponse, TuningProfile};
use std::sync::Arc;
use tracing::info;

use crate::error::{AppError, AppResult};
use crate::state::AppState;

pub async fn register_node(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterNodeRequest>,
) -> AppResult<Json<RegisterNodeResponse>> {
    state
        .store
        .register_node_hardware(
            &payload.node_id,
            &payload.cpu_model,
            payload.cores,
            payload.l2_cache_kb,
            payload.ops_per_sec,
        )
        .await
        .map_err(|e| AppError::Database(sqlx::Error::Protocol(e)))?;

    info!(
        "🖥️ Node Registered: {} | {} | {:.1} M/s",
        payload.node_id,
        payload.cpu_model,
        payload.ops_per_sec / 1_000_000.0
    );

    let strategy = if let Some(l2) = payload.l2_cache_kb {
        if l2 >= 1024 {
            "table"
        } else {
            "fly"
        }
    } else {
        "fly"
    };

    let batch_size = if payload.ops_per_sec > 10_000_000.0 {
        50_000
    } else {
        10_000
    };
    let thread_count = (payload.cores - 1).max(1) as usize;

    Ok(Json(RegisterNodeResponse {
        status: "registered".to_string(),
        tuning: TuningProfile {
            strategy: strategy.to_string(),
            batch_size,
            thread_count,
        },
    }))
}
