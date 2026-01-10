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

/// CLI-specific error types with consistent formatting
#[derive(Error, Debug)]
pub enum CliError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Update failed: {0}")]
    Update(String),

    #[error("Workspace error: {0}")]
    Workspace(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Configuration Error: {0}")] // [Fixed] Added Config variant
    Config(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, CliError>;

impl From<Box<dyn std::error::Error>> for CliError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        CliError::Other(e.to_string())
    }
}

// Implement From<String> for Config if helpful
impl From<String> for CliError {
    fn from(s: String) -> Self {
        CliError::Other(s)
    }
}
