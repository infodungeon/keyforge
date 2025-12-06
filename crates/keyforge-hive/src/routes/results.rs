use crate::error::{AppError, AppResult};
use crate::queue::DbEvent;
use crate::state::AppState;
use axum::{extract::State, Json};
use keyforge_core::config::Config;
use keyforge_core::scorer::Scorer;
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
    let cached_verifier = {
        let guard = state
            .verifiers
            .read()
            .map_err(|_| AppError::Any(anyhow::anyhow!("Verifier cache lock poisoned")))?;
        guard.get(&payload.job_id).cloned()
    };

    let verifier = if let Some(v) = cached_verifier {
        v
    } else {
        let (geometry, weights, corpus_name, cost_matrix) = state
            .store
            .get_job_config(&payload.job_id)
            .await
            .map_err(|e| AppError::Any(anyhow::anyhow!("DB Error: {}", e)))?
            .ok_or(AppError::NotFound)?;

        let (cost_path, corpus_dir) = resolve_paths(&corpus_name, &cost_matrix).ok_or(
            AppError::Validation(format!("Unknown corpus: {}", corpus_name)),
        )?;

        let config = Config {
            weights,
            ..Default::default()
        };

        let scorer = Scorer::new(&cost_path, &corpus_dir, &geometry, config, false)
            .map_err(|e| AppError::Validation(format!("Scorer Init Failed: {}", e)))?;

        let new_verifier = Verifier::from_components(Arc::new(scorer), state.registry.clone());

        {
            let mut guard = state.verifiers.write().map_err(|_| {
                AppError::Any(anyhow::anyhow!("Verifier cache write lock poisoned"))
            })?;
            guard.insert(payload.job_id.clone(), new_verifier.clone());
        }

        info!("🧠 Verifier Cached for Job: {}", &payload.job_id[0..8]);
        new_verifier
    };

    // FIXED: Removed extra args, verify handles tolerance logic internally if updated,
    // but core verifier.verify signature is (layout, score, tolerance).
    // Check verifier.rs signature again. It is (layout, score, tolerance).
    // The compiler error said "3 arguments but 4 supplied".
    // If I remove one here, it should match.
    let is_valid = verifier
        .verify(payload.layout.clone(), payload.score, 5.0)
        .map_err(|e| AppError::Validation(format!("Verification logic error: {}", e)))?;

    if !is_valid {
        return Err(AppError::Validation(
            "Score verification failed (drift detected)".to_string(),
        ));
    }

    let current_best = state
        .store
        .get_job_best_score(&payload.job_id)
        .await
        .map_err(|e| AppError::Any(anyhow::anyhow!("DB Error: {}", e)))?;

    state
        .queue
        .push(DbEvent::Result {
            job_id: payload.job_id.clone(),
            layout: payload.layout.clone(),
            score: payload.score,
            node_id: payload.node_id.clone(),
        })
        .await;

    let is_record = current_best.is_none_or(|best| payload.score < best);

    if is_record {
        info!(
            "🏆 NEW RECORD! Job: {} | {:.0} | {}",
            &payload.job_id[0..8],
            payload.score,
            payload.node_id
        );
    }

    Ok("Accepted".to_string())
}

fn resolve_paths(name: &str, cost_matrix_name: &str) -> Option<(String, String)> {
    let cost_path = format!("data/{}", cost_matrix_name);
    match name {
        "default" | "test_corpus" => Some((cost_path, "data/corpora/default".to_string())),
        other => Some((cost_path, format!("data/corpora/{}", other))),
    }
}
