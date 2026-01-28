// libs/keyforge-model/src/keyboard.rs

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

//! Keyboard aggregate and spatial logic.

use crate::error::ForgeError;
use crate::geometry::KeyNode;
use serde::{Deserialize, Serialize};

/// Pure domain data representing the physical properties of a keyboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyboard {
    /// The list of physical keys.
    pub keys: Vec<KeyNode>,
    /// The logical row index considered the "Home Row".
    pub home_row: crate::types::RowIndex,
    /// Type of keyboard (e.g., "split", "ortho").
    #[serde(default)]
    pub kb_type: String,
}

/// Performance-oriented spatial index for fast distance and origin lookups.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpatialIndex {
    /// Pre-calculated centers for fingers [hand][finger] -> (x, y).
    pub finger_origins: Vec<Vec<(f32, f32)>>,
    /// Pre-calculated squared distances between every pair of physical keys.
    pub spatial_cache: Vec<(f32, f32)>,
}

impl Keyboard {
    /// Creates a new Keyboard definition.
    ///
    /// # Errors
    /// Returns a `ForgeError` if the key list is empty.
    pub fn new(
        keys: Vec<KeyNode>,
        home_row: crate::types::RowIndex,
        kb_type: String,
    ) -> Result<Self, ForgeError> {
        if keys.is_empty() {
            return Err(ForgeError::InvalidData(
                "Keyboard must have at least one key".into(),
            ));
        }

        Ok(Self {
            keys,
            home_row,
            kb_type,
        })
    }

    /// Returns the number of keys on the keyboard.
    #[must_use]
    pub fn count(&self) -> usize {
        self.keys.len()
    }
}

impl SpatialIndex {
    /// Builds a spatial index from a keyboard definition.
    #[must_use]
    pub fn build_from(kb: &Keyboard) -> Self {
        let mut index = Self::default();
        index.calculate_origins(kb);
        index.precompute_spatial_cache(kb);
        index
    }

    fn precompute_spatial_cache(&mut self, kb: &Keyboard) {
        let n = kb.keys.len();
        let mut cache = vec![(0.0, 0.0); n * n];
        for i in 0..n {
            for j in 0..n {
                let dx = kb.keys[i].x - kb.keys[j].x;
                let dy = kb.keys[i].y - kb.keys[j].y;
                cache[i * n + j] = (dx * dx, dy * dy);
            }
        }
        self.spatial_cache = cache;
    }

    fn calculate_origins(&mut self, kb: &Keyboard) {
        let max_hand = kb.keys.iter().map(|k| k.hand.as_usize()).max().unwrap_or(0);
        let max_finger = kb
            .keys
            .iter()
            .map(|k| k.finger.as_usize())
            .max()
            .unwrap_or(0);

        self.finger_origins = vec![vec![(0.0, 0.0); max_finger + 1]; max_hand + 1];

        for hand in 0..=max_hand {
            for finger in 0..=max_finger {
                let origin = kb
                    .keys
                    .iter()
                    .find(|k| {
                        k.hand.as_usize() == hand && k.finger.as_usize() == finger && k.is_home
                    })
                    .or_else(|| {
                        kb.keys.iter().find(|k| {
                            k.hand.as_usize() == hand
                                && k.finger.as_usize() == finger
                                && k.row == kb.home_row
                        })
                    })
                    .or_else(|| {
                        kb.keys
                            .iter()
                            .find(|k| k.hand.as_usize() == hand && k.finger.as_usize() == finger)
                    });

                if let Some(k) = origin {
                    self.finger_origins[hand][finger] = (k.x, k.y);
                }
            }
        }
    }
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;
    use crate::types::{FingerIndex, HandIndex, RowIndex};

    #[test]
    fn test_spatial_index_precomputation() {
        let keys = vec![
            KeyNode {
                index: 0,
                x: 0.0,
                y: 0.0,
                hand: HandIndex(0),
                finger: FingerIndex::new_unchecked(1),
                ..Default::default()
            },
            KeyNode {
                index: 1,
                x: 3.0,
                y: 4.0,
                hand: HandIndex(0),
                finger: FingerIndex::new_unchecked(2),
                ..Default::default()
            },
        ];
        let kb = Keyboard::new(keys, RowIndex(0), "test".into()).unwrap();
        let index = SpatialIndex::build_from(&kb);

        assert_eq!(index.spatial_cache.len(), 4);
        assert_eq!(index.spatial_cache[1], (9.0, 16.0));
    }
}
