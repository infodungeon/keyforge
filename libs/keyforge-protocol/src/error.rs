// libs/keyforge-protocol/src/error.rs

use serde::{Deserialize, Serialize};
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;
use utoipa::ToSchema;

/// Core error codes used across the `KeyForge` system.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    /// Generic internal server error.
    InternalError,
    /// Invalid request payload or configuration.
    ValidationError,
    /// Resource not found (e.g., Job, Asset).
    NotFound,
    /// Authentication or authorization failure.
    Unauthorized,
    /// Rate limit exceeded.
    RateLimited,
    /// Hardware/Calibration failure.
    HardwareError,
    /// Physics engine execution error.
    ComputeError,
    /// Generic bad request.
    BadRequest,
    /// Job configuration failed validation.
    JobValidationFailed,
    /// Upstream dependency timeout or failure.
    UpstreamTimeout,
    /// Database-level failure.
    DatabaseError,
}

/// Unified DTO for error responses across the API and CLI.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct ForgeErrorDto {
    /// Machine-readable error code.
    pub code: ErrorCode,
    /// Human-readable error message.
    pub message: String,
    /// Optional structured details about the error.
    #[cfg_attr(feature = "ts_bindings", ts(type = "any"))]
    pub details: Option<serde_json::Value>,
}

impl ForgeErrorDto {
    /// Creates a new `ForgeErrorDto`.
    #[must_use]
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
        }
    }
}
