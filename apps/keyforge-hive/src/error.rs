use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use keyforge_model::error::ForgeError;
use keyforge_protocol::{ErrorCode, ErrorResponse};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Loader Error: {0}")]
    Loader(#[from] ForgeError),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not Found")]
    NotFound,

    #[error("Internal Server Error: {0}")]
    Any(#[from] anyhow::Error),

    #[error("Service Unavailable: {0}")]
    ServiceUnavailable(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, msg) = match self {
            AppError::Database(e) => {
                tracing::error!("Database Error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorCode::DatabaseError,
                    "A database error occurred".to_string(),
                )
            }
            AppError::Serde(e) => (
                StatusCode::BAD_REQUEST,
                ErrorCode::BadRequest,
                format!("Serialization error: {}", e),
            ),
            AppError::Loader(e) => {
                let status = match &e {
                    ForgeError::NotFound(_) => StatusCode::NOT_FOUND,
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                };
                (status, ErrorCode::InternalError, e.to_string())
            }
            AppError::Validation(s) => (StatusCode::BAD_REQUEST, ErrorCode::JobValidationFailed, s),
            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                ErrorCode::NotFound,
                "Resource not found".to_string(),
            ),
            AppError::Any(e) => {
                tracing::error!("Internal Error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorCode::InternalError,
                    "An internal server error occurred".to_string(),
                )
            }
            AppError::ServiceUnavailable(s) => {
                (StatusCode::SERVICE_UNAVAILABLE, ErrorCode::InternalError, s)
            }
        };

        let body = ErrorResponse::new(code, msg);
        (status, Json(body)).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
