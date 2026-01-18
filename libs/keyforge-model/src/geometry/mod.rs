// libs/keyforge-model/src/geometry/mod.rs

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


//! Physical keyboard geometry and spatial definitions.
//!
//! This module defines the physical properties of a keyboard, including
//! key positions, dimensions, and finger assignments.

use crate::constants::MAX_KEYBOARD_KEYS;
use crate::types::{ColIndex, FingerIndex, HandIndex, KeyIndex, RowIndex};
use crate::validator::Validator;
use crate::asset::{Asset, AssetCategory};
use crate::error::ForgeError;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use utoipa::ToSchema;
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

/// Keyboard Layout Editor (KLE) integration.
pub mod kle;

/// Metadata describing a keyboard definition.
#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct KeyboardMeta {
    /// Display name of the keyboard.
    pub name: String,
    /// Author of the definition.
    #[serde(default)]
    pub author: String,
    /// Version string.
    #[serde(default)]
    pub version: String,
    /// Additional notes or description.
    #[serde(default)]
    pub notes: String,
    /// Type of keyboard (e.g., "split", "ortho").
    #[serde(default, rename = "type")]
    pub kb_type: String,
}

/// Complete definition of a keyboard, including metadata and geometry.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct KeyboardDefinition {
    /// Metadata about the keyboard.
    #[serde(default)]
    pub meta: KeyboardMeta,
    /// Physical geometry of the keys.
    pub geometry: KeyboardGeometry,
    /// Pre-defined layouts available for this keyboard.
    #[serde(default)]
    pub layouts: HashMap<String, String>,
}

impl Asset for KeyboardDefinition {
    fn category() -> AssetCategory {
        AssetCategory::Keyboard
    }

    fn post_load(&mut self) -> Result<(), ForgeError> {
        self.validate().map_err(ForgeError::InvalidData)
    }
}

impl Validator for KeyboardDefinition {
    fn validate(&self) -> Result<(), String> {
        self.geometry.validate()?;
        Ok(())
    }
}

/// Represents a single physical key on the keyboard.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct KeyNode {
    /// Internal numeric ID (0..N) - Runtime optimization.
    #[serde(default)]
    pub index: usize,
    /// Display label / String ID.
    #[serde(default)]
    #[serde(alias = "id")]
    pub label: String,
    /// X coordinate (physical units).
    pub x: f32,
    /// Y coordinate (physical units).
    pub y: f32,
    /// Width (physical units).
    #[serde(default = "default_size")]
    pub w: f32,
    /// Height (physical units).
    #[serde(default = "default_size")]
    pub h: f32,
    /// Rotation angle (degrees).
    #[serde(default)]
    pub r: f32,
    /// Rotation origin X.
    #[serde(default)]
    pub rx: f32,
    /// Rotation origin Y.
    #[serde(default)]
    pub ry: f32,
    /// Hand assignment (Left/Right).
    pub hand: HandIndex,
    /// Finger assignment (Thumb..Pinky).
    pub finger: FingerIndex,
    /// Logical row index.
    #[serde(default)]
    pub row: RowIndex,
    /// Logical column index.
    #[serde(default)]
    pub col: ColIndex,
    /// Whether this is a home row key.
    #[serde(default)]
    pub is_home: bool,
    /// Whether this key requires a stretch to reach.
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

/// Collection of keys and slot definitions defining the keyboard geometry.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct KeyboardGeometry {
    /// List of all physical keys.
    pub keys: Vec<KeyNode>,
    /// Indices of keys considered "Prime" (best positions).
    pub prime_slots: Vec<KeyIndex>,
    /// Indices of keys considered "Medium" quality.
    pub med_slots: Vec<KeyIndex>,
    /// Indices of keys considered "Low" quality.
    pub low_slots: Vec<KeyIndex>,
    /// The logical row index considered the "Home Row".
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
                return Err(format!("Slot index {idx} out of bounds (keys: {max_idx})"));
            }
        }

        for (i, key) in self.keys.iter().enumerate() {
            if key.w <= 0.0 || key.h <= 0.0 { return Err(format!("Key #{i} has invalid dimensions")); }
            if key.hand.as_u8() > 1 { return Err(format!("Key #{} has invalid hand index {}", i, key.hand.as_u8())); }
            if key.finger.as_u8() > 4 { return Err(format!("Key #{} has invalid finger index {}", i, key.finger.as_u8())); }
        }
        Ok(())
    }
}

impl KeyboardDefinition {
    /// Parses a keyboard definition from JSON, supporting both `KeyForge` format and KLE format.
    ///
    /// # Errors
    ///
    /// Returns an error if the content is not valid JSON or KLE format.
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
    /// Returns the total number of keys.
    #[must_use] 
    pub fn key_count(&self) -> usize { self.keys.len() }
}
