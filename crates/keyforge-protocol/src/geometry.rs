use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyboardMeta {
    pub name: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default, rename = "type")]
    pub kb_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardDefinition {
    #[serde(default)]
    pub meta: KeyboardMeta,
    pub geometry: KeyboardGeometry,
    #[serde(default)]
    pub layouts: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyNode {
    #[serde(default)]
    pub id: String,
    pub hand: u8,
    pub finger: u8,
    pub row: i8,
    pub col: i8,
    pub x: f32,
    pub y: f32,

    #[serde(default = "default_size")]
    pub w: f32,
    #[serde(default = "default_size")]
    pub h: f32,

    #[serde(default)]
    pub is_stretch: bool,
}

fn default_size() -> f32 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardGeometry {
    pub keys: Vec<KeyNode>,
    pub prime_slots: Vec<usize>,
    pub med_slots: Vec<usize>,
    pub low_slots: Vec<usize>,
    #[serde(default = "default_home_row")]
    pub home_row: i8,

    #[serde(skip)]
    pub finger_origins: [[(f32, f32); 5]; 2],
}

fn default_home_row() -> i8 {
    1
}

impl Default for KeyboardGeometry {
    fn default() -> Self {
        Self {
            keys: Vec::new(),
            prime_slots: Vec::new(),
            med_slots: Vec::new(),
            low_slots: Vec::new(),
            home_row: 1,
            finger_origins: [[(0.0, 0.0); 5]; 2],
        }
    }
}

impl KeyboardGeometry {
    pub fn key_count(&self) -> usize {
        self.keys.len()
    }

    pub fn calculate_origins(&mut self) {
        self.finger_origins = [[(0.0, 0.0); 5]; 2];
        for hand in 0..2 {
            for finger in 0..5 {
                if let Some(k) = self.keys.iter().find(|k| {
                    k.hand == hand as u8 && k.finger == finger as u8 && k.row == self.home_row
                }) {
                    self.finger_origins[hand][finger] = (k.x, k.y);
                } else if let Some(k) = self
                    .keys
                    .iter()
                    .find(|k| k.hand == hand as u8 && k.finger == finger as u8)
                {
                    self.finger_origins[hand][finger] = (k.x, k.y);
                }
            }
        }
    }
}
