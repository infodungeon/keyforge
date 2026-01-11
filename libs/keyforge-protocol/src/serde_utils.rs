// libs/keyforge-protocol/src/serde_utils.rs

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

//! Serialization and deserialization utilities for the protocol layer.

use keyforge_model::constants::MAX_TRANSPORT_VECTOR_ITEMS;
use serde::{Deserialize, Deserializer};

/// Deserializes a vector with a hard limit on size to prevent memory exhaustion attacks.
/// 
/// This is a Transport Security Policy. It protects the application from processing
/// maliciously large arrays that could cause OOM.
pub fn deserialize_limited_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let v: Vec<T> = Vec::deserialize(deserializer)?;
    if v.len() > MAX_TRANSPORT_VECTOR_ITEMS {
        return Err(serde::de::Error::custom(format!(
            "Vector exceeds transport limit of {} items",
            MAX_TRANSPORT_VECTOR_ITEMS
        )));
    }
    Ok(v)
}
