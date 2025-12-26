use serde::{Deserialize, Serialize};

/// Represents a single physical key.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyNode {
    pub id: usize,     // Internal numeric ID (0..N)
    pub label: String, // For debugging/display
    pub hand: u8,      // 0 = Left, 1 = Right, ... N
    pub finger: u8,    // 0=Thumb, 1=Index, ... N
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
    // Dynamic size to support non-standard hands/fingers (e.g. Datahand, Chorders)
    pub finger_origins: Vec<Vec<(f32, f32)>>,
}

impl Keyboard {
    pub fn new(keys: Vec<KeyNode>, home_row: i8) -> Self {
        let mut kb = Self {
            keys,
            home_row,
            finger_origins: Vec::new(),
        };
        kb.calculate_origins();
        kb
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
