// libs/keyforge-model/src/keyboard.rs

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

//! Keyboard aggregate and spatial logic.

use crate::error::ForgeError;
use crate::geometry::KeyNode;
use serde::{Deserialize, Serialize};

/// The physical reality of the device.
/// Contains the set of keys and pre-calculated spatial data for scoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyboard {
    /// The list of physical keys.
    pub keys: Vec<KeyNode>,
    /// The logical row index considered the "Home Row".
    pub home_row: i8,
    /// Type of keyboard (e.g., "split", "ortho").
    #[serde(default)]
    pub kb_type: String,
    /// Pre-calculated centers for fingers [hand][finger] -> (x, y).
    /// Used for distance calculations relative to the resting position.
    pub finger_origins: Vec<Vec<(f32, f32)>>,
    /// Pre-calculated squared distances between every pair of physical keys.
    /// Index: [i * `key_count` + j] -> (dx^2, dy^2).
    #[serde(skip)]
    pub spatial_cache: Vec<(f32, f32)>,
}

impl Keyboard {
    /// Creates a new Keyboard and pre-calculates finger origins.
    ///
    /// # Errors
    ///
    /// Returns a `ForgeError` if the key list is empty.
    pub fn new(keys: Vec<KeyNode>, home_row: i8, kb_type: String) -> Result<Self, ForgeError> {
        if keys.is_empty() {
            return Err(ForgeError::InvalidData(
                "Keyboard must have at least one key".into(),
            ));
        }

        let mut kb = Self {
            keys,
            home_row,
            kb_type,
            finger_origins: Vec::new(),
            spatial_cache: Vec::new(),
        };
        kb.calculate_origins();

        // Task-prot-rev-017: Validate origins
        for (h_idx, hand) in kb.finger_origins.iter().enumerate() {
            for (f_idx, origin) in hand.iter().enumerate() {
                // If keys exist for this finger but origin is (0,0) and no key is at (0,0)
                let has_keys = kb
                    .keys
                    .iter()
                    .any(|k| k.hand.as_usize() == h_idx && k.finger.as_usize() == f_idx);
                if has_keys && origin.0.abs() < f32::EPSILON && origin.1.abs() < f32::EPSILON {
                    let key_at_zero = kb
                        .keys
                        .iter()
                        .any(|k| k.x.abs() < f32::EPSILON && k.y.abs() < f32::EPSILON);
                    if !key_at_zero {
                        return Err(ForgeError::InvalidData(format!(
                            "Finger origin calculation failed for hand {h_idx}, finger {f_idx}"
                        )));
                    }
                }
            }
        }

        kb.precompute_spatial_cache();
        Ok(kb)
    }

    fn precompute_spatial_cache(&mut self) {
        let n = self.keys.len();
        let mut cache = vec![(0.0, 0.0); n * n];
        for i in 0..n {
            for j in 0..n {
                let dx = self.keys[i].x - self.keys[j].x;
                let dy = self.keys[i].y - self.keys[j].y;
                cache[i * n + j] = (dx * dx, dy * dy);
            }
        }
        self.spatial_cache = cache;
    }

    fn calculate_origins(&mut self) {
        // 1. Determine dimensions
        let max_hand = self
            .keys
            .iter()
            .map(|k| k.hand.as_usize())
            .max()
            .unwrap_or(0);
        let max_finger = self
            .keys
            .iter()
            .map(|k| k.finger.as_usize())
            .max()
            .unwrap_or(0);

        // 2. Initialize with (0,0)
        self.finger_origins = vec![vec![(0.0, 0.0); max_finger + 1]; max_hand + 1];

        // 3. Populate
        for hand in 0..=max_hand {
            for finger in 0..=max_finger {
                // Find Home Row key for this finger
                // Priority 1: Explicit is_home flag
                let origin = self
                    .keys
                    .iter()
                    .find(|k| {
                        k.hand.as_usize() == hand && k.finger.as_usize() == finger && k.is_home
                    })
                    // Priority 2: Match home_row index
                    .or_else(|| {
                        self.keys.iter().find(|k| {
                            k.hand.as_usize() == hand
                                && k.finger.as_usize() == finger
                                && k.row.0 == self.home_row
                        })
                    })
                    // Priority 3: Fallback to any key
                    .or_else(|| {
                        self.keys
                            .iter()
                            .find(|k| k.hand.as_usize() == hand && k.finger.as_usize() == finger)
                    });

                if let Some(k) = origin {
                    self.finger_origins[hand][finger] = (k.x, k.y);
                }
            }
        }
    }

    /// Returns the number of keys on the keyboard.
    #[must_use]
    pub fn count(&self) -> usize {
        self.keys.len()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::types::{FingerIndex, HandIndex, RowIndex};

    #[test]
    fn test_keyboard_spatial_precomputation() {
        let keys = vec![
            KeyNode {
                index: 0,
                x: 0.0,
                y: 0.0,
                hand: HandIndex(0),
                finger: FingerIndex(1),
                ..Default::default()
            },
            KeyNode {
                index: 1,
                x: 1.0,
                y: 0.0,
                hand: HandIndex(0),
                finger: FingerIndex(2),
                ..Default::default()
            },
        ];
        let kb = Keyboard::new(keys, 0, "test".into()).unwrap();

        assert_eq!(kb.spatial_cache.len(), 4);
        // Dist between 0 and 1: dx=1, dy=0 -> (1, 0)
        assert_eq!(kb.spatial_cache[1], (1.0, 0.0));
    }

    #[test]
    fn test_keyboard_basic_methods() {
        assert!(Keyboard::new(vec![], 0, "test".into()).is_err());

        let keys = vec![KeyNode {
            index: 0,
            ..Default::default()
        }];
        let kb = Keyboard::new(keys, 0, "test".into()).unwrap();
        assert_eq!(kb.count(), 1);
    }

    #[test]
    fn test_keyboard_origin_calculation_fallbacks() {
        // Priority 2: Match home_row
        let keys = vec![KeyNode {
            index: 0,
            x: 5.0,
            y: 5.0,
            hand: HandIndex(0),
            finger: FingerIndex(1),
            row: RowIndex(1),
            is_home: false,
            ..Default::default()
        }];
        let kb = Keyboard::new(keys, 1, "test".into()).unwrap();
        assert_eq!(kb.finger_origins[0][1], (5.0, 5.0));

        // Priority 3: Any key
        let keys = vec![KeyNode {
            index: 0,
            x: 7.0,
            y: 7.0,
            hand: HandIndex(0),
            finger: FingerIndex(1),
            row: RowIndex(5),
            is_home: false,
            ..Default::default()
        }];
        let kb = Keyboard::new(keys, 1, "test".into()).unwrap();
        assert_eq!(kb.finger_origins[0][1], (7.0, 7.0));
    }
}
