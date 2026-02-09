// apps/keyforge-agent/src/agent/errors.rs

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

/// A specialized Result type for agent operations.
pub type AgentResult<T> = Result<T, AgentError>;

/// Core error type for the `KeyForge` worker agent.
#[derive(Error, Debug)]
pub enum AgentError {
    /// An error occurred during hardware calibration.
    #[error("Calibration Error: {0}")]
    Calibration(String),

    /// A failure in the physics engine or scoring calculation.
    #[error("Compute Error: {0}")]
    Compute(String),

    /// A network failure while communicating with the Hive server.
    #[error("Network Error: {0}")]
    Network(String),

    /// A WebSocket-specific communication error.
    #[error("WebSocket Error: {0}")]
    WebSocket(String),

    /// Errors related to agent identity, encryption, or key management.
    #[error("Identity Error: {0}")]
    Identity(String),

    /// A required resource was not found.
    #[error("Resource Not Found: {0}")]
    Resource(String),

    /// An error occurred during data serialization or deserialization.
    #[error("Serialization Error: {0}")]
    Serialization(String),

    /// An unexpected internal logic failure.
    #[error("Internal Error: {0}")]
    Internal(String),
}

impl From<std::io::Error> for AgentError {
    fn from(e: std::io::Error) -> Self {
        Self::Internal(e.to_string())
    }
}

impl From<reqwest::Error> for AgentError {
    fn from(e: reqwest::Error) -> Self {
        Self::Network(e.to_string())
    }
}

impl From<serde_json::Error> for AgentError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialization(e.to_string())
    }
}

impl From<toml::de::Error> for AgentError {
    fn from(e: toml::de::Error) -> Self {
        Self::Serialization(e.to_string())
    }
}

impl From<rmp_serde::decode::Error> for AgentError {
    fn from(e: rmp_serde::decode::Error) -> Self {
        Self::Serialization(e.to_string())
    }
}

impl From<url::ParseError> for AgentError {
    fn from(e: url::ParseError) -> Self {
        Self::Network(format!("Invalid URL: {e}"))
    }
}

impl From<keyforge_physics::PhysicsError> for AgentError {
    fn from(e: keyforge_physics::PhysicsError) -> Self {
        Self::Compute(e.to_string())
    }
}

impl From<keyforge_model::error::ForgeError> for AgentError {
    fn from(e: keyforge_model::error::ForgeError) -> Self {
        use keyforge_model::error::ForgeError;
        match e {
            ForgeError::Serialization(s) => Self::Serialization(s),
            ForgeError::Serde(s) => Self::Serialization(s.clone()),
            ForgeError::NotFound(s) => Self::Resource(s),
            ForgeError::InvalidData(s) => Self::Calibration(s),
            _ => Self::Internal(e.to_string()),
        }
    }
}

impl From<keyforge_infra::error::InfraError> for AgentError {
    fn from(e: keyforge_infra::error::InfraError) -> Self {
        use keyforge_infra::error::InfraError;
        match e {
            InfraError::Io(io) => Self::Internal(io.to_string()),
            InfraError::Network(ne) => Self::Network(ne.to_string()),
            InfraError::NetworkString(s) => Self::Network(s),
            InfraError::Serde(se) => Self::Serialization(se.to_string()),
            InfraError::Toml(te) => Self::Internal(te.to_string()),
            InfraError::HashMismatch { expected, actual } => {
                Self::Calibration(format!("Hash mismatch: expected {expected}, got {actual}"))
            }
            InfraError::LockError(s) | InfraError::Config(s) | InfraError::Validation(s) => {
                Self::Internal(s)
            }
            InfraError::Model(e) => Self::from(e),
            InfraError::Internal(s) => Self::Internal(s),
        }
    }
}

impl From<keyforge_model::error::ModelError> for AgentError {
    fn from(e: keyforge_model::error::ModelError) -> Self {
        Self::Internal(e.to_string())
    }
}
