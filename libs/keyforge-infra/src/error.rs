// libs/keyforge-infra/src/error.rs

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

use thiserror::Error;

/// The primary error type for infrastructure-related operations in `KeyForge`.
#[derive(Error, Debug)]
pub enum InfraError {
    /// An error occurred during file I/O or directory management.
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    /// An error occurred during network communication with the Hive.
    #[error("Network Error: {0}")]
    Network(#[from] reqwest::Error),

    /// A network error described by a simple string.
    #[error("Network Error: {0}")]
    NetworkString(String),

    /// An error occurred during JSON serialization or deserialization.
    #[error("Serialization Error: {0}")]
    Serde(#[from] serde_json::Error),

    /// An error occurred during TOML deserialization.
    #[error("TOML Error: {0}")]
    Toml(#[from] toml::de::Error),

    /// A file's SHA-256 hash did not match the expected value.
    #[error("Hash Mismatch: Expected {expected}, Got {actual}")]
    HashMismatch {
        /// The hash value that was expected.
        expected: String,
        /// The actual hash value computed from the content.
        actual: String,
    },

    /// Failed to acquire or release a workspace file lock.
    #[error("Lock Error: {0}")]
    LockError(String),

    /// A configuration error (e.g., malformed URL or missing required asset).
    #[error("Config Error: {0}")]
    Config(String),

    /// Data validation failed during asset loading.
    #[error("Validation Error: {0}")]
    Validation(String),

    /// Error originating from the core domain model.
    #[error("Model Error: {0}")]
    Model(#[from] keyforge_model::error::ForgeError),

    /// An internal or unexpected error occurred.
    #[error("Internal Error: {0}")]
    Internal(String),
}

impl From<keyforge_model::error::ModelError> for InfraError {
    fn from(e: keyforge_model::error::ModelError) -> Self {
        Self::Model(keyforge_model::error::ForgeError::Model(e))
    }
}

impl InfraError {
    /// Returns true if the error is considered transient and should be retried.
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Network(e) => {
                // Retry on timeouts or connection errors or 5xx
                if e.is_timeout() || e.is_connect() {
                    return true;
                }
                if let Some(status) = e.status() {
                    return status.is_server_error()
                        || status == reqwest::StatusCode::TOO_MANY_REQUESTS;
                }
                false
            }
            Self::Io(e) => {
                matches!(
                    e.kind(),
                    std::io::ErrorKind::TimedOut
                        | std::io::ErrorKind::Interrupted
                        | std::io::ErrorKind::WouldBlock
                )
            }
            _ => false,
        }
    }
}

/// A specialized Result type for infrastructure operations.
pub type InfraResult<T> = Result<T, InfraError>;
