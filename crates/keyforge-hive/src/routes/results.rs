// ===== keyforge/crates/keyforge-hive/src/routes/results.rs =====
use crate::error::{AppError, AppResult};
use crate::queue::DbEvent;
use crate::state::AppState;
use axum::{extract::State, Json};
use keyforge_core::config::Config;
use keyforge_core::verifier::Verifier;
use serde::Deserialize;
use std::sync::Arc;
use tracing::info;

#[derive(Deserialize)]
pub struct SubmitResultRequest {
    pub job_id: String,
    pub layout: String,
    pub score: f32,
    pub node_id: String,
}

pub async fn submit(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SubmitResultRequest>,
) -> AppResult<String> {
    // 1. Fetch Job Config
    let (geometry, weights, corpus_name, cost_matrix) = state
        .store
        .get_job_config(&payload.job_id)
        .await
        .map_err(|e| AppError::Any(anyhow::anyhow!("DB Error: {}", e)))?
        .ok_or(AppError::NotFound)?;

    // 2. Resolve Paths
    let (cost_path, corpus_dir) = resolve_paths(&corpus_name, &cost_matrix).ok_or(
        AppError::Validation(format!("Unknown corpus: {}", corpus_name)),
    )?;

    // 3. Initialize Verifier
    let config = Config {
        weights,
        ..Default::default()
    };

    let verifier = Verifier::new(
        &cost_path,
        &corpus_dir, // Now a directory path
        &geometry,
        config,
        "data/keycodes.json",
    )
    .map_err(|e| AppError::Validation(format!("Verifier Init Failed: {}", e)))?;

    // 4. Verify Score
    let is_valid = verifier
        .verify(payload.layout.clone(), payload.score, 5.0)
        .map_err(|e| AppError::Validation(format!("Verification logic error: {}", e)))?;

    if !is_valid {
        return Err(AppError::Validation(
            "Score verification failed (drift detected)".to_string(),
        ));
    }

    // 5. Check for Record
    let current_best = state
        .store
        .get_job_best_score(&payload.job_id)
        .await
        .map_err(|e| AppError::Any(anyhow::anyhow!("DB Error: {}", e)))?;

    // 6. Persist
    state
        .queue
        .push(DbEvent::Result {
            job_id: payload.job_id.clone(),
            layout: payload.layout.clone(),
            score: payload.score,
            node_id: payload.node_id.clone(),
        })
        .await;

    // 7. Log
    let is_record = current_best.is_none_or(|best| payload.score < best);

    if is_record {
        info!(
            "🏆 NEW RECORD! Job: {} | {:.0} | {}",
            &payload.job_id[0..8],
            payload.score,
            payload.node_id
        );
    } else {
        info!(
            "📥 Contribution: Job: {} | {:.0}",
            &payload.job_id[0..8],
            payload.score
        );
    }

    Ok("Accepted".to_string())
}

fn resolve_paths(name: &str, cost_matrix_name: &str) -> Option<(String, String)> {
    let cost_path = format!("data/{}", cost_matrix_name);

    match name {
        "default" | "test_corpus" => Some((cost_path, "data/corpora/default".to_string())),
        // Map other names to directories
        other => Some((cost_path, format!("data/corpora/{}", other))),
    }
}
