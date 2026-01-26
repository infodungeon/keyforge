// apps/keyforge-hive/src/api/analysis.rs

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::{extract::State, Json};
use keyforge_protocol::AnalysisReportDto;
use std::sync::Arc;

/// POST /analysis/validate
/// Validates a layout string and returns analysis.
pub(crate) async fn validate_layout(
    State(_state): State<Arc<AppState>>,
    Json(_payload): Json<serde_json::Value>,
) -> AppResult<Json<AnalysisReportDto>> {
    Err(AppError::Internal("Not fully implemented".into()))
}
