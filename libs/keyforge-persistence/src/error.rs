// libs/keyforge-persistence/src/error.rs

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

#[derive(Error, Debug)]
pub enum PersistenceError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization Error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Project not found: {0}")]
    ProjectNotFound(String),

    #[error("Invalid project state: {0}")]
    InvalidState(String),

    #[error("Asset Load Error: {0}")]
    AssetLoad(String),

    #[error("Validation Error: {0}")]
    Validation(String),

    /// Errors during domain translation.
    #[error("Adapter error: {0}")]
    Adapter(String),

    /// Errors from the domain layer.
    #[error("Domain error: {0}")]
    Forge(#[from] keyforge_model::error::ForgeError),
}

pub type PersistenceResult<T> = Result<T, PersistenceError>;
