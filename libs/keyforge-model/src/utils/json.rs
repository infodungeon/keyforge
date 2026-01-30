// libs/keyforge-model/src/utils/json.rs

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

//! Unified JSON serialization utilities.

use crate::error::ForgeError;
use serde::{de::DeserializeOwned, Serialize};

/// Safe serialization wrapper.
///
/// # Errors
/// Returns `ForgeError::Serde` if serialization fails.
pub fn to_string_safe<T: Serialize>(value: &T) -> Result<String, ForgeError> {
    serde_json::to_string(value).map_err(|e| ForgeError::Serde(e.to_string()))
}

/// Safe pretty-serialization wrapper.
///
/// # Errors
/// Returns `ForgeError::Serde` if serialization fails.
pub fn to_string_pretty_safe<T: Serialize>(value: &T) -> Result<String, ForgeError> {
    serde_json::to_string_pretty(value).map_err(|e| ForgeError::Serde(e.to_string()))
}

/// Safe deserialization wrapper.
///
/// # Errors
/// Returns `ForgeError::Serde` if parsing fails.
pub fn from_str_safe<T: DeserializeOwned>(s: &str) -> Result<T, ForgeError> {
    serde_json::from_str(s).map_err(|e| ForgeError::Serde(e.to_string()))
}