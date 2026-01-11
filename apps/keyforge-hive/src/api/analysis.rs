// apps/keyforge-hive/src/api/analysis.rs

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


use crate::error::{AppError, AppResult};

use crate::models::ValidationResult;
use crate::state::AppState;
use axum::{extract::State, Json};
use keyforge_adapter::conversion;
use keyforge_core::loader::AssetLoader;
use keyforge_model::ScoringWeights;
use serde::Deserialize;
use std::sync::Arc;

/// Request payload for performing a quick on-demand analysis of a keyboard layout.
#[derive(Deserialize)]
pub struct ValidateRequest {
    /// The serialized layout string to analyze.
    pub layout_str: String,
    /// Optional scoring weights to use. Defaults to standard weights if omitted.
    pub weights: Option<ScoringWeights>,
    /// Optional name of the keyboard geometry to use. Defaults to "ortho_30".
    pub keyboard_name: Option<String>,
    /// Optional list of corpus sources. Defaults to "text/en_std" if omitted.
    pub corpus_sources: Option<Vec<keyforge_model::CorpusSource>>,
    /// Optional cost overrides. Defaults to empty if omitted.
    pub cost_overrides: Option<Vec<(usize, usize, f32)>>,
}

use keyforge_model::constants::{DEFAULT_KEYBOARD_ID, DEFAULT_CORPUS_ID, ASSET_KEYCODES_FILENAME, DEFAULT_CORPUS_WEIGHT};

/// Performs a quick scoring analysis of a layout against a standard corpus.
/// This endpoint does not register a job or persist results.
pub async fn validate_layout(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ValidateRequest>,
) -> AppResult<Json<ValidationResult>> {
    // Use provided keyboard name or fallback
    let keyboard_name = payload.keyboard_name.as_deref().unwrap_or(DEFAULT_KEYBOARD_ID);

    let corpus_sources = payload.corpus_sources.clone().unwrap_or_else(|| vec![keyforge_model::CorpusSource {
        id: DEFAULT_CORPUS_ID.to_string(),
        weight: DEFAULT_CORPUS_WEIGHT,
        hash: None,
    }]);

    // Load assets from the server’s global cache (implements AssetLoader)
    let definition = state
        .assets
        .load_keyboard(keyboard_name)
        .await
        .map_err(|e| AppError::Validation(format!("Keyboard load failed: {}", e)))?;

    let registry = state
        .assets
        .load_keycodes(ASSET_KEYCODES_FILENAME)
        .await
        .map_err(|e| AppError::Validation(format!("Keycodes load failed: {}", e)))?;

    let corpus = state
        .assets
        .load_corpus(&corpus_sources)
        .await
        .map_err(|e| AppError::Validation(format!("Corpus load failed: {}", e)))?;

    // Determine scoring weights (defaults if not provided)
    let weights = payload.weights.unwrap_or_default();

    // definition.geometry is MODEL.
    // engine needs MODEL.
    // So we can use it directly via Keyboard::new.
    let domain_keyboard = keyforge_model::Keyboard::new(
        definition.geometry.keys.clone(),
        definition.geometry.home_row,
    ).map_err(|e| AppError::Validation(format!("Invalid keyboard: {}", e)))?;

    let domain_rubric = conversion::to_domain_rubric(&weights);

    let cost_overrides = payload.cost_overrides.unwrap_or_default();

    let engine = keyforge_core::build_engine(&keyforge_core::EngineRequest {
        keyboard: Arc::new(domain_keyboard),
        corpus: corpus,
        rubric: Arc::new(domain_rubric),
        config: keyforge_model::SearchConfig::default(),
        initial_layout: None,
        pinned_keys: vec![],
        cost_overrides,
    })
    .map_err(|e| AppError::Validation(format!("Failed to build physics engine: {}", e)))?;

    let key_count = engine.key_count();
    let layout = conversion::parse_layout_string(&payload.layout_str, key_count, &registry)
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let report = engine.analyze(&layout)
        .map_err(|e| AppError::Validation(format!("Analysis failed: {}", e)))?;

    // Model geometry is now standard
    let proto_geometry = definition.geometry.clone();

    Ok(Json(ValidationResult {
        layout_name: "Custom".to_string(),
        score: report,
        geometry: proto_geometry,
        heatmap: vec![0.0; key_count],
        penalty_map: vec![],
    }))
}