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

#[derive(Error, Debug)]
pub enum InfraError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Network Error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Serialization Error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Hash Mismatch: Expected {expected}, Got {actual}")]
    HashMismatch { expected: String, actual: String },

    #[error("Lock Error: {0}")]
    LockError(String),

    #[error("Config Error: {0}")]
    Config(String),

    #[error("Validation Error: {0}")]
    Validation(String),
}

pub type InfraResult<T> = Result<T, InfraError>;
