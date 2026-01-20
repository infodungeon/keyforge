// libs/keyforge-model/src/layout.rs

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

//! Layout entity and related logic.
//!
//! A `Layout` represents a complete mapping of logical `KeyCode`s to physical
//! `KeyIndex` positions on a keyboard.

pub use crate::types::{KeyCode, KeyIndex};
use serde::{Deserialize, Serialize};
use thiserror::Error;
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

/// Errors related to Layout construction and validation.
#[derive(Error, Debug)]
pub enum LayoutError {
    /// Layout contains the same key code multiple times.
    #[error("Layout contains duplicate keys")]
    DuplicateKeys,
}

/// A specific mapping of `KeyCodes` to physical positions.
/// The index in the vector corresponds to the `KeyIndex`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct Layout {
    /// The list of keys.
    /// The index corresponds to the `KeyIndex`.
    #[cfg_attr(feature = "ts_bindings", ts(type = "Vec<KeyCode>"))]
    pub keys: Vec<KeyCode>,
}

impl Layout {
    /// Creates a layout without validation.
    /// Use `try_from` for safe construction.
    #[must_use]
    pub fn new_unchecked(keys: Vec<KeyCode>) -> Self {
        Self { keys }
    }

    /// Returns the number of keys in the layout.
    #[must_use]
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Returns true if the layout has no keys.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
}

impl TryFrom<Vec<KeyCode>> for Layout {
    type Error = LayoutError;

    fn try_from(keys: Vec<KeyCode>) -> Result<Self, Self::Error> {
        // Validation Logic: Duplicates are now allowed (e.g. for split spacebars).
        Ok(Self { keys })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::HashSet;

    #[test]
    fn test_layout_basic_methods() {
        let keys = vec![KeyCode(65), KeyCode(66)];
        let layout = Layout::new_unchecked(keys.clone());
        assert_eq!(layout.len(), 2);
        assert!(!layout.is_empty());
        assert_eq!(layout.keys, keys);

        let empty = Layout::new_unchecked(vec![]);
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn test_layout_validation() {
        // Duplicates are allowed
        let keys = vec![KeyCode(65), KeyCode(66), KeyCode(65)];
        assert!(Layout::try_from(keys).is_ok());

        // Valid
        let keys = vec![KeyCode(65), KeyCode(66), KeyCode(67)];
        assert!(Layout::try_from(keys).is_ok());
    }

    proptest! {
        #[test]
        fn test_layout_validity(keys in prop::collection::vec(0u16..100, 0..50)) {
            let key_codes: Vec<KeyCode> = keys.into_iter().map(KeyCode).collect();
            let result = Layout::try_from(key_codes);
            prop_assert!(result.is_ok());
        }
    }
}
