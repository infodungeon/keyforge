use serde::{Deserialize, Serialize};
use crate::error::ForgeError;
// use crate::types::{HandIndex, FingerIndex, RowIndex, ColIndex}; // Removed unused imports

use crate::geometry::KeyNode;

/// The physical reality of the device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyboard {
    pub keys: Vec<KeyNode>,
    pub home_row: i8,
    // Pre-calculated centers for fingers [hand][finger] -> (x, y)
    pub finger_origins: Vec<Vec<(f32, f32)>>,
}

impl Keyboard {
    pub fn new(keys: Vec<KeyNode>, home_row: i8) -> Result<Self, ForgeError> {
        if keys.is_empty() {
            return Err(ForgeError::InvalidData("Keyboard must have at least one key".into()));
        }

        // Validation is now implicit via Types, but we can check bounds if needed.
        // HandIndex/FingerIndex guarantee valid ranges at construction.

        let mut kb = Self {
            keys,
            home_row,
            finger_origins: Vec::new(),
        };
        kb.calculate_origins();
        Ok(kb)
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
                let origin = self
                    .keys
                    .iter()
                    .find(|k| {
                        k.hand.as_usize() == hand && k.finger.as_usize() == finger && k.row.0 == self.home_row
                    })
                    .or_else(|| {
                        // Fallback: Find *any* key for this finger if home row missing
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

    pub fn count(&self) -> usize {
        self.keys.len()
    }
}
