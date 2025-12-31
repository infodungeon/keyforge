// Copyright (c) 2025 KeyForge Contributors
//
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

use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use thiserror::Error;
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;
pub use crate::types::{KeyCode, KeyIndex};

/// Errors related to Layout construction and validation.
#[derive(Error, Debug)]
pub enum LayoutError {
    /// Layout contains the same key code multiple times.
    #[error("Layout contains duplicate keys")]
    DuplicateKeys,
}

/// A specific mapping of KeyCodes to physical positions.
/// The index in the vector corresponds to the `KeyIndex`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct Layout {
    /// The list of keys.
    /// Optimization: Store up to 64 keys inline on the stack.
    #[cfg_attr(feature = "ts_bindings", ts(type = "Vec<KeyCode>"))]
    pub keys: SmallVec<[KeyCode; 64]>,
}

impl Layout {
    /// Creates a layout without validation.
    /// Use `try_from` for safe construction.
    pub fn new_unchecked(keys: Vec<KeyCode>) -> Self {
        Self {
            keys: SmallVec::from_vec(keys),
        }
    }

    /// Returns the number of keys in the layout.
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Returns true if the layout has no keys.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
}

impl TryFrom<Vec<KeyCode>> for Layout {
    type Error = LayoutError;

    fn try_from(keys: Vec<KeyCode>) -> Result<Self, Self::Error> {
        // Validation Logic: Check for duplicates
        for i in 0..keys.len() {
            for j in (i + 1)..keys.len() {
                if keys[i] == keys[j] {
                    return Err(LayoutError::DuplicateKeys);
                }
            }
        }

        Ok(Self {
            keys: SmallVec::from_vec(keys),
        })
    }
}
