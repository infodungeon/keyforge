// Copyright (c) 2025 KeyForge Contributors
//
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
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use utoipa::ToSchema;

/// Standardized error codes for the KeyForge API.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, Display, EnumString,
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
    /// Creates a new ErrorResponse.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
        }
    }

    /// Adds details to the ErrorResponse.
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}
