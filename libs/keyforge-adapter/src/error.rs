// libs/keyforge-adapter/src/error.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use thiserror::Error;

/// Errors that can occur during domain conversion or protocol translation.
#[derive(Debug, Error, PartialEq)]
pub enum AdapterError {
    /// Provided data failed validation constraints.
    #[error("Validation failed: {0}")]
    Validation(String),

    /// A key label or token was not found in the registry.
    #[error("Unknown key token: {0}")]
    UnknownToken(String),

    /// The layout string length exceeds the safety limit.
    #[error("Layout string exceeds maximum length of {0}")]
    LayoutTooLong(usize),
}

/// A specialized result type for adapter operations.
pub type AdapterResult<T> = Result<T, AdapterError>;
