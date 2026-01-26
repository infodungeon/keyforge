// apps/keyforge-hive/src/api/analysis.rs

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    Json,
};
use keyforge_adapter::loader::AssetLoader;
use keyforge_model::KeyboardDefinition;
use keyforge_protocol::{AnalysisReportDto, JobConfig};
use std::sync::Arc;

/// GET /api/keyboards/{name}/analysis
/// Returns detailed ergonomic analysis for a given layout on a keyboard.
#[utoipa::path(
    get,
    path = "/api/keyboards/{keyboard_name}/analysis",
    params(
        ("keyboard_name" = String, Path, description = "Keyboard definition ID"),
        ("layout" = String, Query, description = "Space-separated layout string")
    ),
    responses(
        (status = 200, description = "Analysis report", body = AnalysisReportDto)
    ),
    tag = "analysis"
)]
#[allow(dead_code)]
pub(crate) async fn analyze_layout(
    State(state): State<Arc<AppState>>,
    Path(keyboard_name): Path<String>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> AppResult<Json<AnalysisReportDto>> {
    let layout_str = params.get("layout").ok_or(AppError::NotFound)?;

    let definition = state
        .assets
        .load::<KeyboardDefinition>(&keyboard_name)
        .await
        .map_err(|_| AppError::NotFound)?;

    let registry = state
        .assets
        .load::<keyforge_model::KeycodeRegistry>("keycodes.json")
        .await
        .unwrap_or_else(|_| Arc::new(keyforge_model::KeycodeRegistry::new_with_defaults()));

    let weights = keyforge_model::config::ScoringWeights::default();
    let corpus_sources = vec![keyforge_model::config::CorpusSource::default()];

    let job_config = JobConfig {
        definition: definition.as_ref().clone().into(),
        weights: weights.clone().into(),
        params: keyforge_model::config::SearchParams::default().into(),
        pinned_keys: vec![].into(),
        corpora: corpus_sources
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>()
            .into(),
        cost_matrix: keyforge_model::config::CostMatrixSource::default().into(),
        biometrics: vec![].into(),
        parent_job_id: None,
        baseline_score: None,
        parents: vec![].into(),
    };

    let builder = keyforge_compute::SessionBuilder::new(state.assets.as_ref())
        .with_keyboard_def(definition)
        .with_corpus(&job_config.to_domain_corpus_sources())
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .with_cost_matrix(&job_config.to_domain_cost_matrix())
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .with_keycodes("keycodes.json")
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .with_rubric(keyforge_adapter::conversion::to_domain_rubric(
            &job_config.to_domain_weights(),
        ));

    let session = builder
        .build()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let parsed = keyforge_adapter::conversion::parse_layout_string(
        layout_str,
        session.engine.key_count(),
        &registry,
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let report = session
        .engine
        .analyze(&parsed)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(report.into()))
}

/// POST /analysis/validate
/// Validates a layout string and returns analysis.
pub(crate) async fn validate_layout(
    State(_state): State<Arc<AppState>>,
    Json(_payload): Json<serde_json::Value>,
) -> AppResult<Json<AnalysisReportDto>> {
    Err(AppError::Internal("Not fully implemented".into()))
}
