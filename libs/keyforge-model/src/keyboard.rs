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

use serde::{Deserialize, Serialize};
use crate::error::ForgeError;
use crate::geometry::KeyNode;

/// The physical reality of the device.
/// Contains the set of keys and pre-calculated spatial data for scoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyboard {
    /// The list of physical keys.
    pub keys: Vec<KeyNode>,
    /// The logical row index considered the "Home Row".
    pub home_row: i8,
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
    pub fn new(keys: Vec<KeyNode>, home_row: i8) -> Result<Self, ForgeError> {
        if keys.is_empty() {
            return Err(ForgeError::InvalidData("Keyboard must have at least one key".into()));
        }

        let mut kb = Self {
            keys,
            home_row,
            finger_origins: Vec::new(),
            spatial_cache: Vec::new(),
        };
        kb.calculate_origins();
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
        let max_hand = self.keys.iter().map(|k| k.hand.as_usize()).max().unwrap_or(0);
        let max_finger = self.keys.iter().map(|k| k.finger.as_usize()).max().unwrap_or(0);

        // 2. Initialize with (0,0)
        self.finger_origins = vec![vec![(0.0, 0.0); max_finger + 1]; max_hand + 1];

        // 3. Populate
        for hand in 0..=max_hand {
            for finger in 0..=max_finger {
                // Find Home Row key for this finger
                // Priority 1: Explicit is_home flag
                let origin = self.keys.iter().find(|k| {
                    k.hand.as_usize() == hand && k.finger.as_usize() == finger && k.is_home
                })
                // Priority 2: Match home_row index
                .or_else(|| self.keys.iter().find(|k| {
                    k.hand.as_usize() == hand && k.finger.as_usize() == finger && k.row.0 == self.home_row
                }))
                // Priority 3: Fallback to any key
                .or_else(|| self.keys.iter().find(|k| {
                    k.hand.as_usize() == hand && k.finger.as_usize() == finger
                }));

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
