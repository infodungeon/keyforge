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

use keyforge_model::error::ForgeError;
use thiserror::Error;

/// Errors that can occur during persistence operations.
#[derive(Error, Debug)]
pub enum PersistenceError {
    /// Error propagated from the asset loader.
    #[error("Loader Error: {0}")]
    Loader(#[from] ForgeError),

    /// Standard IO error.
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    /// Error during JSON (de)serialization.
    #[error("Serialization Error: {0}")]
    Serde(#[from] serde_json::Error),

    /// Configuration or logic error.
    #[error("Config Error: {0}")]
    Config(String),

    /// Internal error, often related to concurrency (e.g., poisoned mutex).
    #[error("Internal Error: {0}")]
    Internal(String),
}

/// A specialized Result type for persistence operations.
pub type PersistenceResult<T> = Result<T, PersistenceError>;
