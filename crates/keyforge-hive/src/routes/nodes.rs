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
    // 1. Register in DB
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

    // 2. Intelligent Tuning Logic

    // Strategy: If L2 cache is small (< 512KB per core approx) or unknown, be conservative.
    let strategy = if let Some(l2) = payload.l2_cache_kb {
        if l2 >= 1024 {
            "table"
        } else {
            "fly"
        }
    } else {
        "fly" // Safe default
    };

    // Batch Size: If CPU is fast, run longer batches to amortize sync overhead
    let batch_size = if payload.ops_per_sec > 10_000_000.0 {
        50_000
    } else if payload.ops_per_sec > 4_000_000.0 {
        20_000
    } else {
        5_000
    };

    // Thread Count: Reserve 1 core for OS/Network if possible
    let thread_count = if payload.cores > 2 {
        (payload.cores - 1) as usize
    } else {
        1
    };

    Ok(Json(RegisterNodeResponse {
        status: "registered".to_string(),
        tuning: TuningProfile {
            strategy: strategy.to_string(),
            batch_size,
            thread_count, // FIXED: Added missing field
        },
    }))
}
