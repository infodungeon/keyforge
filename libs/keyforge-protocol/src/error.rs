// libs/keyforge-protocol/src/error.rs

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

//! Standardized error types for the protocol.

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};
use utoipa::ToSchema;

/// Standardized error codes for the `KeyForge` API.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    ToSchema,
    Display,
    EnumString,
    EnumIter,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    // Generic
    /// Internal server error.
    InternalError,
    /// Bad request (validation failed).
    BadRequest,
    /// Resource not found.
    NotFound,

    // Auth
    /// Authentication token missing.
    AuthMissing,
    /// Authentication token invalid.
    AuthInvalid,
    /// Authentication token expired.
    AuthExpired,
    /// Access forbidden.
    AuthForbidden,

    // Domain
    /// Job validation failed.
    JobValidationFailed,
    /// Job not found.
    JobNotFound,
    /// Job already exists.
    JobAlreadyExists,

    // Infrastructure
    /// Database error.
    DatabaseError,
    /// Upstream service timeout.
    UpstreamTimeout,
    /// Service unavailable.
    ServiceUnavailable,
}

/// Standardized error response structure.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    /// The error code.
    pub code: ErrorCode,
    /// A human-readable message.
    pub message: String,
    /// Optional details (JSON).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ErrorResponse {
    /// Creates a new `ErrorResponse`.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
        }
    }

    /// Adds details to the `ErrorResponse`.
    #[must_use]
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_error_code_conversions() {
        assert_eq!(ErrorCode::InternalError.to_string(), "INTERNAL_ERROR");
        assert_eq!(
            ErrorCode::from_str("INTERNAL_ERROR").unwrap(),
            ErrorCode::InternalError
        );
        assert!(ErrorCode::from_str("INVALID").is_err());
    }

    #[test]
    fn test_error_codes_exhaustive() {
        use strum::IntoEnumIterator;
        for code in ErrorCode::iter() {
            let s = code.to_string();
            assert_eq!(ErrorCode::from_str(&s).unwrap(), code);

            let json = serde_json::to_string(&code).unwrap();
            assert_eq!(json, format!("\"{s}\""));
        }
    }

    #[test]
    fn test_error_response_builder() {
        let err = ErrorResponse::new(ErrorCode::BadRequest, "test message")
            .with_details(serde_json::json!({"foo": "bar"}));

        assert_eq!(err.code, ErrorCode::BadRequest);
        assert_eq!(err.message, "test message");
        assert_eq!(err.details.unwrap()["foo"], "bar");
    }
}
