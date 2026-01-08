// libs/keyforge-model/src/serde_utils.rs

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


//! Serialization and deserialization utilities.

use serde::{Deserialize, Deserializer};

/// Deserializes a vector with a hard limit on size to prevent memory exhaustion attacks.
pub fn deserialize_limited_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let v: Vec<T> = Vec::deserialize(deserializer)?;
    // Hard limit of 100k items to prevent memory exhaustion
    if v.len() > 100_000 {
        return Err(serde::de::Error::custom(
            "Vector exceeds limit of 100,000 items",
        ));
    }
    Ok(v)
}

/// Helper for serde skip_serializing_if to satisfy ts-rs parser.
pub fn is_none<T>(option: &Option<T>) -> bool {
    option.is_none()
}
