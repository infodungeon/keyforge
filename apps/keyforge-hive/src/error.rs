// apps/keyforge-hive/src/error.rs

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

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use keyforge_model::error::ForgeError;
use keyforge_protocol::{ErrorCode, ForgeErrorDto};
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

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal Server Error: {0}")]
    Any(#[from] anyhow::Error),

    #[error("Internal Error: {0}")]
    Internal(String),

    #[error("Service Unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Configuration Error: {0}")]
    Config(String),
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
                format!("Serialization error: {e}"),
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
            AppError::Conflict(s) => (StatusCode::CONFLICT, ErrorCode::BadRequest, s),
            AppError::Any(e) => {
                tracing::error!("Internal Error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorCode::InternalError,
                    "An internal server error occurred".to_string(),
                )
            }
            AppError::Internal(s) => {
                tracing::error!("Internal Error: {}", s);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorCode::InternalError,
                    s,
                )
            }
            // Startups errors like Config typically panic before HTTP, but if returned:
            AppError::Config(s) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::InternalError,
                s,
            ),
            AppError::ServiceUnavailable(s) => {
                (StatusCode::SERVICE_UNAVAILABLE, ErrorCode::InternalError, s)
            }
        };

        let body = ForgeErrorDto::new(code, msg);
        (status, Json(body)).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
