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
    /// Errors occurring during filesystem or stream I/O.
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    /// Errors occurring during JSON or other serialization formats.
    #[error("Serialization Error: {0}")]
    Serde(#[from] serde_json::Error),

    /// Errors occurring during binary serialization with postcard.
    #[error("Postcard Error: {0}")]
    Postcard(#[from] postcard::Error),

    /// A project or session file could not be found.
    #[error("Project not found: {0}")]
    ProjectNotFound(String),

    /// The requested operation is invalid for the current state.
    #[error("Invalid project state: {0}")]
    InvalidState(String),

    /// Failed to load a required asset from persistence.
    #[error("Asset Load Error: {0}")]
    AssetLoad(String),

    /// Data integrity or validation failure.
    #[error("Validation Error: {0}")]
    Validation(String),

    /// Errors during translation between domain and persistence models.
    #[error("Adapter error: {0}")]
    Adapter(String),

    /// Re-wrapped errors from the domain layer.
    #[error("Domain error: {0}")]
    Forge(#[from] keyforge_model::error::ForgeError),

    /// Errors originating from internal task spawning or joining.
    #[error("Internal Task Error: {0}")]
    Task(String),
}

pub type PersistenceResult<T> = Result<T, PersistenceError>;
