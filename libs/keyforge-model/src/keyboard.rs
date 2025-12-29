use serde::{Deserialize, Serialize};
use crate::error::KeyForgeError;

/// Represents a single physical key.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyNode {
    pub id: usize,     // Internal numeric ID (0..N)
    pub label: String, // For debugging/display
    pub hand: u8,      // 0 = Left, 1 = Right
    pub finger: u8,    // 0=Thumb, 1=Index, 2=Middle, 3=Ring, 4=Pinky
    pub row: i8,
    pub col: i8,
    pub x: f32,
    pub y: f32,
    pub is_home: bool,
}

/// The physical reality of the device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyboard {
    pub keys: Vec<KeyNode>,
    pub home_row: i8,
    // Pre-calculated centers for fingers [hand][finger] -> (x, y)
    pub finger_origins: Vec<Vec<(f32, f32)>>,
}

impl Keyboard {
    pub fn new(keys: Vec<KeyNode>, home_row: i8) -> Result<Self, KeyForgeError> {
        if keys.is_empty() {
            return Err(KeyForgeError::InvalidData("Keyboard must have at least one key".into()));
        }

        for (i, key) in keys.iter().enumerate() {
            if key.hand > 1 {
                return Err(KeyForgeError::InvalidData(format!(
                    "Key #{} has invalid hand index {} (must be 0 or 1)",
                    i, key.hand
                )));
            }
            if key.finger > 4 {
                return Err(KeyForgeError::InvalidData(format!(
                    "Key #{} has invalid finger index {} (must be 0-4)",
                    i, key.finger
                )));
            }
        }

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
        let max_hand = self.keys.iter().map(|k| k.hand).max().unwrap_or(0) as usize;
        let max_finger = self.keys.iter().map(|k| k.finger).max().unwrap_or(0) as usize;

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
                        k.hand == hand as u8 && k.finger == finger as u8 && k.row == self.home_row
                    })
                    .or_else(|| {
                        // Fallback: Find *any* key for this finger if home row missing
                        self.keys
                            .iter()
                            .find(|k| k.hand == hand as u8 && k.finger == finger as u8)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_keyboard() {
        let keys = vec![KeyNode {
            id: 0,
            label: "A".into(),
            hand: 0,
            finger: 1,
            row: 0,
            col: 0,
            x: 0.0,
            y: 0.0,
            is_home: false,
        }];
        assert!(Keyboard::new(keys, 0).is_ok());
    }

    #[test]
    fn test_empty_keys_error() {
        assert!(Keyboard::new(vec![], 0).is_err());
    }

    #[test]
    fn test_invalid_hand_error() {
        let keys = vec![KeyNode {
            id: 0,
            label: "A".into(),
            hand: 2,
            finger: 1,
            row: 0,
            col: 0,
            x: 0.0,
            y: 0.0,
            is_home: false,
        }];
        assert!(Keyboard::new(keys, 0).is_err());
    }
}
