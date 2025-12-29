use crate::constants::MAX_KEYBOARD_KEYS;
use crate::types::{ColIndex, FingerIndex, HandIndex, KeyIndex, RowIndex};
use crate::validator::Validator;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use utoipa::ToSchema;

pub mod kle;

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct KeyboardDefinition {
    #[serde(default)]
    pub meta: KeyboardMeta,
    pub geometry: KeyboardGeometry,
    #[serde(default)]
    pub layouts: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct KeyNode {
    /// Internal numeric ID (0..N) - Runtime optimization
    #[serde(default)]
    pub index: usize,
    /// Display label / String ID
    #[serde(default)]
    pub label: String,
    
    // Position & Dimensions
    pub x: f32,
    pub y: f32,
    #[serde(default = "default_size")]
    pub w: f32,
    #[serde(default = "default_size")]
    pub h: f32,
    #[serde(default)]
    pub r: f32,
    #[serde(default)]
    pub rx: f32,
    #[serde(default)]
    pub ry: f32,
    
    // Assignment
    pub hand: HandIndex,
    pub finger: FingerIndex,
    #[serde(default)]
    pub row: RowIndex,
    #[serde(default)]
    pub col: ColIndex,
    
    // Flags
    #[serde(default)]
    pub is_home: bool,
    #[serde(default)]
    pub is_stretch: bool,
}

impl Default for KeyNode {
    fn default() -> Self {
        Self {
            index: 0,
            label: String::new(),
            x: 0.0,
            y: 0.0,
            w: 1.0,
            h: 1.0,
            r: 0.0,
            rx: 0.0,
            ry: 0.0,
            hand: HandIndex::default(),
            finger: FingerIndex::default(),
            row: RowIndex::default(),
            col: ColIndex::default(),
            is_home: false,
            is_stretch: false,
        }
    }
}

fn default_size() -> f32 { 1.0 }
fn default_home_row() -> i8 { 1 }

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct KeyboardGeometry {
    pub keys: Vec<KeyNode>,
    pub prime_slots: Vec<KeyIndex>,
    pub med_slots: Vec<KeyIndex>,
    pub low_slots: Vec<KeyIndex>,
    #[serde(default = "default_home_row")]
    pub home_row: i8,
}

impl Validator for KeyboardGeometry {
    fn validate(&self) -> Result<(), String> {
        if self.keys.is_empty() {
            return Err("Keyboard geometry must have at least one key".to_string());
        }
        if self.keys.len() > MAX_KEYBOARD_KEYS {
            return Err(format!("Too many keys: {} (Max: {})", self.keys.len(), MAX_KEYBOARD_KEYS));
        }

        let prime: HashSet<_> = self.prime_slots.iter().collect();
        let med: HashSet<_> = self.med_slots.iter().collect();
        let low: HashSet<_> = self.low_slots.iter().collect();

        if !prime.is_disjoint(&med) { return Err("Prime and Med slots overlap".to_string()); }
        if !prime.is_disjoint(&low) { return Err("Prime and Low slots overlap".to_string()); }
        if !med.is_disjoint(&low) { return Err("Med and Low slots overlap".to_string()); }

        let total_slots = prime.len() + med.len() + low.len();
        if total_slots != self.keys.len() {
            return Err(format!("Slot definition incomplete. Covered {}/{} keys.", total_slots, self.keys.len()));
        }

        let max_idx = self.keys.len();
        for &idx in self.prime_slots.iter().chain(&self.med_slots).chain(&self.low_slots) {
            if (idx.0 as usize) >= max_idx {
                return Err(format!("Slot index {} out of bounds (keys: {})", idx, max_idx));
            }
        }

        for (i, key) in self.keys.iter().enumerate() {
            if key.w <= 0.0 || key.h <= 0.0 { return Err(format!("Key #{} has invalid dimensions", i)); }
            if key.hand.as_u8() > 1 { return Err(format!("Key #{} has invalid hand index {}", i, key.hand.as_u8())); }
            if key.finger.as_u8() > 4 { return Err(format!("Key #{} has invalid finger index {}", i, key.finger.as_u8())); }
        }
        Ok(())
    }
}

impl KeyboardDefinition {
    pub fn parse(content: &str, name_hint: Option<&str>) -> Result<Self, String> {
        if let Ok(def) = serde_json::from_str::<KeyboardDefinition>(content) {
            return Ok(def);
        }
        if let Ok(geom) = kle::parse_kle_json(content) {
            let name = name_hint.unwrap_or("Imported Board").to_string();
            return Ok(KeyboardDefinition {
                meta: KeyboardMeta {
                    name,
                    author: "Imported from KLE".to_string(),
                    kb_type: "imported".to_string(),
                    ..Default::default()
                },
                geometry: geom,
                layouts: HashMap::new(),
            });
        }
        Err("Failed to parse keyboard JSON".to_string())
    }
}

impl Default for KeyboardGeometry {
    fn default() -> Self {
        Self {
            keys: Vec::new(),
            prime_slots: Vec::new(),
            med_slots: Vec::new(),
            low_slots: Vec::new(),
            home_row: 1,
        }
    }
}

impl KeyboardGeometry {
    pub fn key_count(&self) -> usize { self.keys.len() }
}
