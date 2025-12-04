// ===== keyforge/crates/keyforge-hive/src/routes/nodes.rs =====
use axum::{extract::State, Json};
use keyforge_core::protocol::{RegisterNodeRequest, RegisterNodeResponse, TuningProfile};
use std::sync::Arc;
use tracing::info;

use crate::error::{AppError, AppResult};
use crate::state::AppState;

pub async fn register_node(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterNodeRequest>,
) -> AppResult<Json<RegisterNodeResponse>> {
    // 1. Register in DB (Updates hardware_profiles and nodes tables)
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
        // FIXED: Removed redundant .into()
        .map_err(|e| AppError::Database(sqlx::Error::Protocol(e)))?;

    info!(
        "🖥️ Node Registered: {} | {} | {:.1} M/s",
        payload.node_id,
        payload.cpu_model,
        payload.ops_per_sec / 1_000_000.0
    );

    // 2. Determine Tuning Strategy
    // Heuristic: If L2 cache is small (< 512KB per core approx) or unknown, be conservative.
    // Or if the CPU is very fast, we can use larger batches.

    let strategy = if let Some(l2) = payload.l2_cache_kb {
        if l2 >= 1024 {
            "table"
        } else {
            "fly"
        }
    } else {
        "fly" // Safe default
    };

    let batch_size = if payload.ops_per_sec > 10_000_000.0 {
        50_000
    } else {
        10_000
    };

    Ok(Json(RegisterNodeResponse {
        status: "registered".to_string(),
        tuning: TuningProfile {
            strategy: strategy.to_string(),
            batch_size,
        },
    }))
}
