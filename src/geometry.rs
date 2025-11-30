use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// --- METADATA ---
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyboardMeta {
    pub name: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub notes: String,

    // NEW: "row_staggered", "column_staggered", "ortho"
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
    #[serde(default)]
    pub is_stretch: bool,
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

impl KeyboardDefinition {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("❌ Failed to read keyboard file: {}", e))?;

        let mut def: KeyboardDefinition = serde_json::from_str(&content)
            .map_err(|e| format!("❌ Failed to parse keyboard JSON: {}", e))?;

        let key_count = def.geometry.keys.len();
        for (name, layout_str) in &def.layouts {
            if layout_str.chars().count() != key_count {
                return Err(format!(
                    "❌ Layout '{}' length ({}) does not match Geometry key count ({})",
                    name,
                    layout_str.chars().count(),
                    key_count
                ));
            }
        }

        def.geometry.calculate_origins();
        Ok(def)
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
