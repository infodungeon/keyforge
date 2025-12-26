use crate::error::{AppError, AppResult};
use crate::models::ValidationResult;
use crate::state::AppState;
use axum::{extract::State, Json};
use keyforge_adapter::conversion;
use keyforge_model::loader::AssetLoader;
use keyforge_protocol::config::{CorpusSource, ScoringWeights};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct ValidateRequest {
    pub layout_str: String,
    pub weights: Option<ScoringWeights>,
    pub keyboard_name: Option<String>,
}

pub async fn validate_layout(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ValidateRequest>,
) -> AppResult<Json<ValidationResult>> {
    // Use provided keyboard name or fallback
    let keyboard_name = payload.keyboard_name.as_deref().unwrap_or("ortho_30");

    // NOTE: this endpoint is effectively a “quick analysis” endpoint.
    // For now it uses a fixed corpus (same as previous behavior).
    let corpus_sources = [CorpusSource {
        id: "text/en_std".to_string(),
        weight: 1.0,
        hash: None,
    }];

    // Load assets from the server’s global cache (implements AssetLoader)
    let definition = state
        .assets
        .load_keyboard(keyboard_name)
        .map_err(|e| AppError::Validation(format!("Keyboard load failed: {}", e)))?;

    let registry = state
        .assets
        .load_keycodes("keycodes.json")
        .map_err(|e| AppError::Validation(format!("Keycodes load failed: {}", e)))?;

    let corpus = state
        .assets
        .load_corpus(&corpus_sources)
        .map_err(|e| AppError::Validation(format!("Corpus load failed: {}", e)))?;

    // Determine scoring weights (defaults if not provided)
    let weights = payload.weights.unwrap_or_default();

    // Convert protocol structures into domain structures
    let domain_keyboard = conversion::to_domain_keyboard(&definition.geometry);
    let domain_rubric = conversion::to_domain_rubric(&weights);

    // No custom cost matrix overrides for this endpoint
    let cost_overrides: Vec<(usize, usize, f32)> = Vec::new();

    let engine = keyforge_core::build_engine(&keyforge_core::EngineRequest {
        keyboard: std::sync::Arc::new(domain_keyboard),
        corpus: std::sync::Arc::new(corpus),
        rubric: std::sync::Arc::new(domain_rubric),
        config: keyforge_model::SearchConfig::default(),
        initial_layout: None,
        pinned_keys: vec![],
        cost_overrides,
    });

    let key_count = engine.key_count();
    let layout = conversion::parse_layout_string(&payload.layout_str, key_count, &registry)
        .map_err(AppError::Validation)?;

    let report = engine.analyze(&layout);

    Ok(Json(ValidationResult {
        layout_name: "Custom".to_string(),
        score: report,
        geometry: definition.geometry.clone(),
        heatmap: vec![0.0; key_count],
        penalty_map: vec![],
    }))
}
