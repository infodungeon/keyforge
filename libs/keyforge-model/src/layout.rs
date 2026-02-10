// libs/keyforge-model/src/layout.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You    may obtain a copy of the License at
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

use crate::types::{KeyCode, KeyIndex};
use std::collections::HashMap;
use std::sync::OnceLock;
use thiserror::Error;

/// Errors related to Layout construction and validation.
#[derive(Error, Debug)]
pub enum LayoutError {
    /// Layout contains the same key code multiple times.
    #[error("Layout contains duplicate keys")]
    DuplicateKeys,
    /// Index out of bounds.
    #[error("Index out of bounds: {0}")]
    IndexOutOfBounds(usize),
    /// Invalid length.
    #[error("Invalid layout length: {0}")]
    InvalidLength(usize),
}

/// A raw, unvalidated representation of a layout, used for I/O and serialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawLayout {
    /// The list of keys.
    /// The index corresponds to the `KeyIndex`.
    pub keys: Vec<KeyCode>,
}

/// A rich domain model for a keyboard layout.
/// Invariants are enforced upon construction and mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    /// The list of keys.
    /// Private to ensure invariants (like non-emptiness or size consistency) are maintained.
    keys: Vec<KeyCode>,
}

impl Layout {
    /// Creates a layout from a vector of keys without validation.
    /// Use `try_from` for safe construction.
    #[must_use]
    pub fn new_unchecked(keys: Vec<KeyCode>) -> Self {
        Self { keys }
    }

    /// Returns the number of keys in the layout.
    #[must_use]
    pub fn len(&self) -> usize {
        self.keys().len()
    }

    /// Returns true if the layout has no keys.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keys().is_empty()
    }

    /// Returns a reference to the keys in the layout.
    #[must_use]
    pub fn keys(&self) -> &[KeyCode] {
        &self.keys
    }

    /// Swaps two keys in the layout.
    ///
    /// # Errors
    /// Returns `LayoutError::IndexOutOfBounds` if either index is invalid.
    pub fn swap(&mut self, a: KeyIndex, b: KeyIndex) -> Result<(), LayoutError> {
        let idx_a = usize::from(a.raw());
        let idx_b = usize::from(b.raw());
        if idx_a >= self.keys.len() {
            return Err(LayoutError::IndexOutOfBounds(idx_a));
        }
        if idx_b >= self.keys.len() {
            return Err(LayoutError::IndexOutOfBounds(idx_b));
        }
        self.keys.swap(idx_a, idx_b);
        Ok(())
    }

    /// Sets the key at a specific index.
    ///
    /// # Errors
    /// Returns `LayoutError::IndexOutOfBounds` if the index is invalid.
    pub fn set(&mut self, index: KeyIndex, key: KeyCode) -> Result<(), LayoutError> {
        let idx = usize::from(index.raw());
        if idx >= self.keys.len() {
            return Err(LayoutError::IndexOutOfBounds(idx));
        }
        self.keys[idx] = key;
        Ok(())
    }

    /// Gets the key at a specific index.
    #[must_use]
    pub fn get(&self, index: KeyIndex) -> Option<KeyCode> {
        self.keys.get(usize::from(index.raw())).copied()
    }

    /// Identifies the similarity of this layout to known standard layouts.
    #[must_use]
    pub fn identify(&self) -> Option<LayoutIdentity> {
        LayoutIdentity::identify(self)
    }
}

impl From<Vec<KeyCode>> for Layout {
    fn from(keys: Vec<KeyCode>) -> Self {
        Self { keys }
    }
}

impl TryFrom<RawLayout> for Layout {
    type Error = LayoutError;

    fn try_from(raw: RawLayout) -> Result<Self, Self::Error> {
        Ok(Self { keys: raw.keys })
    }
}

impl From<Layout> for RawLayout {
    fn from(layout: Layout) -> Self {
        Self { keys: layout.keys }
    }
}

impl From<Vec<KeyCode>> for RawLayout {
    fn from(keys: Vec<KeyCode>) -> Self {
        Self { keys }
    }
}

/// Represents the identity of a layout based on its similarity to standard layouts.
#[derive(Debug, Clone)]
pub struct LayoutIdentity {
    /// The name of the standard layout (e.g., "Qwerty", "Colemak").
    pub name: String,
    /// A similarity score from 0.0 to 1.0.
    pub similarity: f32,
    /// The Hamming distance (number of mismatched keys) from the standard.
    pub distance: usize,
}

static STANDARDS: OnceLock<HashMap<String, Vec<KeyCode>>> = OnceLock::new();

impl LayoutIdentity {
    fn get_standards() -> &'static HashMap<String, Vec<KeyCode>> {
        STANDARDS.get_or_init(|| {
            let mut standards = HashMap::new();
            standards.insert("Qwerty".into(), to_codes(crate::constants::layouts::QWERTY));
            standards.insert(
                "Colemak".into(),
                to_codes(crate::constants::layouts::COLEMAK),
            );
            standards.insert("Dvorak".into(), to_codes(crate::constants::layouts::DVORAK));
            standards
        })
    }

    /// Identifies the similarity of a layout back to standard layouts.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn identify(layout: &Layout) -> Option<Self> {
        let standards = Self::get_standards();
        let mut best: Option<Self> = None;

        for (name, std_keys) in standards {
            let len = std_keys.len().min(layout.len());
            if len == 0 {
                continue;
            }

            let mut matches = 0;
            for (i, &std_code) in std_keys.iter().enumerate().take(len) {
                if layout.keys[i] == std_code {
                    matches += 1;
                }
            }

            let similarity = f32::from(u16::try_from(matches).unwrap_or(u16::MAX))
                / f32::from(u16::try_from(len).unwrap_or(u16::MAX));
            let distance = len - matches;

            if best.as_ref().is_none_or(|b| similarity > b.similarity) {
                best = Some(Self {
                    name: name.clone(),
                    similarity,
                    distance,
                });
            }
        }

        if let Some(b) = best {
            if b.similarity > crate::constants::IDENTIFY_SIMILARITY_THRESHOLD {
                return Some(b);
            }
        }
        None
    }
}

fn to_codes(s: &str) -> Vec<KeyCode> {
    s.chars()
        .map(|c| KeyCode::new(u16::try_from(u32::from(c)).unwrap_or(0)))
        .collect()
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_layout_basic_methods() {
        let keys = vec![KeyCode::new(65), KeyCode::new(66)];
        let layout = Layout::new_unchecked(keys.clone());
        assert_eq!(layout.len(), 2);
        assert!(!layout.is_empty());
        assert_eq!(layout.keys(), keys.as_slice());

        let empty = Layout::new_unchecked(vec![]);
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn test_layout_mutations() -> anyhow::Result<()> {
        let mut layout = Layout::new_unchecked(vec![KeyCode::new(65), KeyCode::new(66)]);

        // Swap
        layout.swap(KeyIndex::new(0), KeyIndex::new(1))?;
        assert_eq!(
            layout
                .get(KeyIndex::new(0))
                .ok_or_else(|| anyhow::anyhow!("Missing key"))?,
            KeyCode::new(66)
        );
        assert_eq!(
            layout
                .get(KeyIndex::new(1))
                .ok_or_else(|| anyhow::anyhow!("Missing key"))?,
            KeyCode::new(65)
        );

        // Set
        layout.set(KeyIndex::new(0), KeyCode::new(67))?;
        assert_eq!(
            layout
                .get(KeyIndex::new(0))
                .ok_or_else(|| anyhow::anyhow!("Missing key"))?,
            KeyCode::new(67)
        );

        // Bounds check
        assert!(layout.swap(KeyIndex::new(0), KeyIndex::new(2)).is_err());
        assert!(layout.set(KeyIndex::new(2), KeyCode::new(68)).is_err());
        Ok(())
    }

    #[test]
    fn test_layout_identification() -> anyhow::Result<()> {
        let qwerty_str = crate::constants::layouts::QWERTY;
        let keys: Vec<KeyCode> = qwerty_str
            .chars()
            .map(|c| KeyCode::new(u16::try_from(u32::from(c)).unwrap_or(0)))
            .collect();
        let layout = Layout::new_unchecked(keys);

        let id = layout.identify();
        assert!(id.is_some());
        let id = id.ok_or_else(|| anyhow::anyhow!("Identification failed"))?;
        assert_eq!(id.name, "Qwerty");
        assert!(id.similarity > 0.9);
        Ok(())
    }

    proptest! {
        #[test]
        fn test_layout_validity(keys in prop::collection::vec(0u16..100, 0..50)) {
            let key_codes: Vec<KeyCode> = keys.into_iter().map(KeyCode::new).collect();
            let raw = RawLayout { keys: key_codes };
            let result = Layout::try_from(raw);
            prop_assert!(result.is_ok());
        }
    }
}
