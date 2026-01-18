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

///
/// # Errors
/// Returns a deserialization error if the vector length exceeds the transport limit.
pub fn deserialize_limited_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let v: Vec<T> = Vec::deserialize(deserializer)?;
    if v.len() > MAX_TRANSPORT_VECTOR_ITEMS {
        return Err(serde::de::Error::custom(format!(
            "Vector exceeds transport limit of {MAX_TRANSPORT_VECTOR_ITEMS} items"
        )));
    }
    Ok(v)
}
