use axum::{extract::State, http::StatusCode, Json};
use keyforge_core::{
    config::Config, layouts::layout_string_to_u16, optimizer::mutation, scorer::Scorer,
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::{info, warn};

use crate::state::AppState;

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
) -> Result<String, (StatusCode, String)> {
    // 1. Fetch Job Configuration from Store
    let (geometry, weights, corpus_name) = state
        .store
        .get_job_config(&payload.job_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?
        .ok_or((StatusCode::NOT_FOUND, "Job not found".to_string()))?;

    // 2. Resolve Data Files (Cost Matrix / Ngrams)
    let (cost_path, ngram_path) = resolve_paths(&corpus_name).ok_or((
        StatusCode::BAD_REQUEST,
        format!("Unknown corpus: {}", corpus_name),
    ))?;

    // 3. Initialize Scorer (Verification)
    let config = Config {
        weights,
        ..Default::default()
    };
    let scorer = Scorer::new(&cost_path, &ngram_path, &geometry, config, false).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Scorer Init Failed: {}", e),
        )
    })?;

    // 4. Validate Score
    let key_count = geometry.keys.len();

    // Convert the submitted string back to u16 codes using the server's registry
    let layout_codes = layout_string_to_u16(&payload.layout, key_count, &state.registry);

    // Build the position map (O(1) lookup)
    let pos_map = mutation::build_pos_map(&layout_codes);

    // Calculate the score locally
    let details = scorer.score_details(&pos_map, 3000);

    // Check for drift/cheating (Tolerance: 5.0 points)
    if (details.layout_score - payload.score).abs() > 5.0 {
        warn!(
            "⚠️ Score mismatch for Job {}. Claimed: {:.2}, Calculated: {:.2}",
            &payload.job_id[0..8],
            payload.score,
            details.layout_score
        );
        return Err((StatusCode::BAD_REQUEST, "Score verification failed".into()));
    }

    // 5. Check for Record (for logging purposes)
    let current_best = state
        .store
        .get_job_best_score(&payload.job_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // 6. Persist Result
    state
        .store
        .save_result(
            &payload.job_id,
            &payload.layout,
            payload.score,
            &payload.node_id,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // 7. Log Interaction
    // FIXED: Use is_none_or per clippy suggestion
    let is_record = current_best.is_none_or(|best| payload.score < best);

    if is_record {
        info!(
            "🏆 NEW RECORD! Job: {} | Score: {:.0} | Node: {}",
            &payload.job_id[0..8],
            payload.score,
            payload.node_id
        );
    } else {
        info!(
            "📥 Contribution: Job: {} | Score: {:.0} | Node: {}",
            &payload.job_id[0..8],
            payload.score,
            payload.node_id
        );
    }

    Ok("Accepted".to_string())
}

/// Resolves abstract corpus names to concrete file paths.
fn resolve_paths(name: &str) -> Option<(String, String)> {
    match name {
        "default" | "test_corpus" => Some((
            "data/cost_matrix.csv".to_string(),
            "data/ngrams-all.tsv".to_string(),
        )),
        _ => None,
    }
}
