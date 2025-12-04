use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    // FIXED: Suppress warning for currently unused standard error variant
    #[allow(dead_code)]
    #[error("Not Found")]
    NotFound,

    #[error("Internal Server Error: {0}")]
    Any(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            AppError::Database(e) => {
                tracing::error!("Database Error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error".to_string(),
                )
            }
            AppError::Serde(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            AppError::Validation(s) => (StatusCode::BAD_REQUEST, s),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Resource not found".to_string()),
            AppError::Any(e) => {
                tracing::error!("Internal Error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        };

        (status, Json(json!({ "error": msg }))).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
