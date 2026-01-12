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
use keyforge_core::loader::AssetLoader;
use keyforge_model::ScoringWeights;
use keyforge_protocol::JobConfig;
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

use keyforge_model::constants::{DEFAULT_KEYBOARD_ID, DEFAULT_CORPUS_ID, DEFAULT_CORPUS_WEIGHT};
use crate::services::runner::AgentRunner;

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

    // Load definitions to ensure they exist, but we will pass config to Runner
    let definition = state
        .assets
        .load_keyboard(keyboard_name)
        .await
        .map_err(|e| AppError::Validation(format!("Keyboard load failed: {}", e)))?;

    // Determine scoring weights (defaults if not provided)
    let weights = payload.weights.unwrap_or_default();
    
    // Construct JobConfig for Runner
    let job_config = JobConfig {
        definition: definition.as_ref().clone(),
        weights: weights,
        params: keyforge_model::config::SearchParams::default(),
        pinned_keys: vec![],
        corpora: corpus_sources,
        cost_matrix: keyforge_model::CostMatrixSource::Predefined("cost_matrix.json".into()), // Simplified default
        biometrics: vec![],
        parent_job_id: None,
        baseline_score: None,
        parents: vec![],
    };

    // Initialize Runner
    let runner = AgentRunner::new(state.data_path.clone());
    
    // Delegate to Agent Sidecar
    let json_output = runner.run_validation(&job_config, &payload.layout_str).await?;
    
    // Deserialize report
    let report: keyforge_model::AnalysisReport = serde_json::from_str(&json_output)
        .map_err(|e| AppError::Any(anyhow::anyhow!("Failed to parse agent output: {}", e)))?;

    Ok(Json(ValidationResult {
        layout_name: "Custom".to_string(),
        score: report.clone(),
        geometry: definition.geometry.clone(),
        heatmap: report.heatmap,
        penalty_map: report.penalty_map,
    }))
}