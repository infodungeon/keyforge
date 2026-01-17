// apps/keyforge-cli/src/error.rs

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
use keyforge_adapter::AdapterError;
use keyforge_physics::PhysicsError;
use keyforge_evolution::EvolutionError;

/// CLI-specific error types with consistent formatting
#[derive(Error, Debug)]
pub enum CliError {
    /// Network request failure.
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// Self-update failure.
    #[error("Update failed: {0}")]
    Update(String),

    /// Workspace or filesystem error.
    #[error("Workspace error: {0}")]
    Workspace(String),

    /// Input/Output error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::error::Error),

    /// Adapter error.
    #[error("Adapter error: {0}")]
    Adapter(#[from] AdapterError),

    /// Physics error.
    #[error("Physics error: {0}")]
    Physics(#[from] PhysicsError),

    /// Evolution error.
    #[error("Evolution error: {0}")]
    Evolution(#[from] EvolutionError),

    /// Generic or miscellaneous error.
    #[error("{0}")]
    Other(String),
}

/// Result alias for CLI operations.
pub type Result<T> = std::result::Result<T, CliError>;

impl From<Box<dyn std::error::Error>> for CliError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        CliError::Other(e.to_string())
    }
}

impl From<anyhow::Error> for CliError {
    fn from(e: anyhow::Error) -> Self {
        CliError::Other(e.to_string())
    }
}
