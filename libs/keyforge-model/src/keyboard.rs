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
use crate::types::{Movement, Point};

/// The physical reality of the device.
/// Contains the set of keys and pre-calculated spatial data for scoring.
#[derive(Debug, Clone)]
pub struct Keyboard {
    /// The list of physical keys.
    pub keys: Vec<KeyNode>,
    /// The logical row index considered the "Home Row".
    pub home_row: crate::types::RowIndex,
    /// Type of keyboard (e.g., "split", "ortho").
    pub kb_type: String,
    /// Pre-calculated centers for fingers \[hand\]\[finger\] -> Point.
    /// Used for distance calculations relative to the resting position.
    pub finger_origins: Vec<Vec<Point>>,
    /// Pre-calculated movements between every pair of physical keys.
    /// Index: [i * `key_count` + j] -> Movement.
    pub spatial_cache: Vec<Movement>,
}

impl Keyboard {
    /// Creates a new Keyboard and pre-calculates finger origins.
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

        let mut kb = Self {
            keys,
            home_row,
            kb_type,
            finger_origins: Vec::new(),
            spatial_cache: Vec::new(),
        };
        kb.calculate_origins();

        // Validation of origins
        for (h_idx, hand) in kb.finger_origins.iter().enumerate() {
            for (f_idx, origin) in hand.iter().enumerate() {
                let has_keys = kb
                    .keys
                    .iter()
                    .any(|k| k.hand.as_usize() == h_idx && k.finger.as_usize() == f_idx);
                if has_keys && origin.x.raw() == 0 && origin.y.raw() == 0 {
                    let key_at_zero = kb.keys.iter().any(|k| k.x.raw() == 0 && k.y.raw() == 0);
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

    /// Returns a reference to the physical keys.
    #[must_use]
    pub fn keys(&self) -> &[KeyNode] {
        &self.keys
    }

    /// Returns the home row index.
    #[must_use]
    pub fn home_row(&self) -> crate::types::RowIndex {
        self.home_row
    }

    fn precompute_spatial_cache(&mut self) {
        let n = self.keys.len();
        let mut cache = vec![Movement::default(); n * n];
        for i in 0..n {
            let k1 = &self.keys[i];
            for j in 0..n {
                let k2 = &self.keys[j];
                cache[i * n + j] = Movement::from_keys(k1, k2);
            }
        }
        self.spatial_cache = cache;
    }

    fn calculate_origins(&mut self) {
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

        self.finger_origins = vec![vec![Point::default(); max_finger + 1]; max_hand + 1];

        for hand in 0..=max_hand {
            for finger in 0..=max_finger {
                let origin = self
                    .keys
                    .iter()
                    .find(|k| {
                        k.hand.as_usize() == hand && k.finger.as_usize() == finger && k.is_home
                    })
                    .or_else(|| {
                        self.keys.iter().find(|k| {
                            k.hand.as_usize() == hand
                                && k.finger.as_usize() == finger
                                && k.row == self.home_row
                        })
                    })
                    .or_else(|| {
                        self.keys
                            .iter()
                            .find(|k| k.hand.as_usize() == hand && k.finger.as_usize() == finger)
                    });

                if let Some(k) = origin {
                    self.finger_origins[hand][finger] = Point::new(k.x, k.y);
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

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;
    use crate::types::{FingerIndex, HandIndex, KeyIndex};

    #[test]
    fn test_keyboard_spatial_precomputation() -> anyhow::Result<()> {
        let keys = vec![
            KeyNode {
                index: KeyIndex(0),
                x: crate::types::SpatialUnit::from_f32(0.0),
                y: crate::types::SpatialUnit::from_f32(0.0),
                hand: HandIndex::new(0),
                finger: FingerIndex::INDEX,
                ..Default::default()
            },
            KeyNode {
                index: KeyIndex(1),
                x: crate::types::SpatialUnit::from_f32(3.0),
                y: crate::types::SpatialUnit::from_f32(4.0),
                hand: HandIndex::new(0),
                finger: FingerIndex::MIDDLE,
                ..Default::default()
            },
        ];
        let kb = Keyboard::new(keys, crate::types::RowIndex::new(0), "test".into())?;

        assert_eq!(kb.spatial_cache.len(), 4);
        // (3-0)^2 + (4-0)^2 = 9 + 16 = 25
        assert_eq!(kb.spatial_cache[1].dist_sq(), 25_000_000);
        Ok(())
    }
}
