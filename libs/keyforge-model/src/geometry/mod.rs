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

use crate::asset::{Asset, AssetCategory};
use crate::constants::{MAX_KEYBOARD_KEYS, MAX_KEYBOARD_NAME_LEN};
use crate::error::ForgeError;
use crate::types::{ColIndex, FingerIndex, HandIndex, KeyIndex, RowIndex};
use crate::validator::Validator;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;
use utoipa::ToSchema;

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
        if self.meta.name.len() > MAX_KEYBOARD_NAME_LEN {
            return Err(format!("Keyboard name too long (max {MAX_KEYBOARD_NAME_LEN})"));
        }
        self.geometry.validate()
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

fn default_size() -> f32 {
    1.0
}

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
    pub home_row: i8,
}

impl Default for KeyboardGeometry {
    fn default() -> Self {
        Self {
            keys: Vec::new(),
            prime_slots: Vec::new(),
            med_slots: Vec::new(),
            low_slots: Vec::new(),
            home_row: 1, // Default fallback
        }
    }
}

impl Validator for KeyboardGeometry {
    fn validate(&self) -> Result<(), String> {
        if self.keys.is_empty() {
            return Err("Keyboard geometry must have at least one key".to_string());
        }

        // Task-prot-rev-011: Validate home row against keys
        let has_home_keys = self.keys.iter().any(|k| k.is_home);
        let has_home_row_matches = self.keys.iter().any(|k| k.row.0 == self.home_row);

        if !has_home_keys && !has_home_row_matches {
            return Err(format!(
                "Invalid home row {}: no keys found on this row and no keys marked is_home",
                self.home_row
            ));
        }

        if self.keys.len() > MAX_KEYBOARD_KEYS {
            return Err(format!(
                "Too many keys: {} (Max: {})",
                self.keys.len(),
                MAX_KEYBOARD_KEYS
            ));
        }

        let prime: HashSet<_> = self.prime_slots.iter().collect();
        let med: HashSet<_> = self.med_slots.iter().collect();
        let low: HashSet<_> = self.low_slots.iter().collect();

        if !prime.is_disjoint(&med) {
            return Err("Prime and Med slots overlap".to_string());
        }
        if !prime.is_disjoint(&low) {
            return Err("Prime and Low slots overlap".to_string());
        }
        if !med.is_disjoint(&low) {
            return Err("Med and Low slots overlap".to_string());
        }

        let total_slots = prime.len() + med.len() + low.len();
        if total_slots != self.keys.len() {
            return Err(format!(
                "Slot definition incomplete. Covered {}/{} keys.",
                total_slots,
                self.keys.len()
            ));
        }

        let max_idx = self.keys.len();
        for &idx in self
            .prime_slots
            .iter()
            .chain(&self.med_slots)
            .chain(&self.low_slots)
        {
            if (idx.0 as usize) >= max_idx {
                return Err(format!("Slot index {idx} out of bounds (keys: {max_idx})"));
            }
        }

        for (i, key) in self.keys.iter().enumerate() {
            if key.w <= 0.0 || key.h <= 0.0 {
                return Err(format!("Key #{i} has invalid dimensions"));
            }
            if key.hand.as_u8() > 1 {
                return Err(format!(
                    "Key #{} has invalid hand index {}",
                    i,
                    key.hand.as_u8()
                ));
            }
            if key.finger.as_u8() > 4 {
                return Err(format!(
                    "Key #{} has invalid finger index {}",
                    i,
                    key.finger.as_u8()
                ));
            }
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

impl KeyboardGeometry {
    /// Returns the total number of keys.
    #[must_use]
    pub fn key_count(&self) -> usize {
        self.keys.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_geometry_validation() {
        let mut geom = KeyboardGeometry::default();
        geom.home_row = 0; // Match KeyNode::default() row
        // 1. Empty keys
        assert!(geom.validate().is_err());

        // 2. Invalid Key dimensions
        geom.keys.push(KeyNode {
            index: 0,
            hand: HandIndex(0),
            finger: FingerIndex(1),
            w: 0.0,
            ..Default::default()
        });
        geom.prime_slots.push(KeyIndex(0));
        assert!(geom.validate().is_err(), "Should fail on w=0");

        // 3. Slot overlaps
        geom.keys = vec![
            KeyNode {
                index: 0,
                hand: HandIndex(0),
                finger: FingerIndex(1),
                w: 1.0,
                h: 1.0,
                ..Default::default()
            },
            KeyNode {
                index: 1,
                hand: HandIndex(1),
                finger: FingerIndex(1),
                w: 1.0,
                h: 1.0,
                ..Default::default()
            },
        ];
        geom.prime_slots = vec![KeyIndex(0)];
        geom.med_slots = vec![KeyIndex(0)]; // Overlap with prime
        geom.low_slots = vec![KeyIndex(1)];
        assert!(geom.validate().is_err(), "Should fail on slot overlap");

        // 4. Out of bounds slot index
        geom.keys = vec![KeyNode::default(); 3];
        geom.prime_slots = vec![KeyIndex(0)];
        geom.med_slots = vec![KeyIndex(1)];
        geom.low_slots = vec![KeyIndex(99)]; // Out of bounds
        let res = geom.validate();
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("out of bounds"));

        // 5. Incomplete slot definition
        geom.low_slots = vec![]; // Key 2 is missing
        let res = geom.validate();
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Slot definition incomplete"));

        // 6. Invalid hand/finger via new_unchecked
        geom.low_slots = vec![KeyIndex(2)];
        geom.keys[0].hand = HandIndex(5);
        assert!(geom.validate().is_err());
        geom.keys[0].hand = HandIndex(0);
        geom.keys[0].finger = FingerIndex::new_unchecked(10);
        assert!(geom.validate().is_err());
        geom.keys[0].finger = FingerIndex::new_unchecked(1);

        // 7. Valid state
        assert!(geom.validate().is_ok());
    }

    #[test]
    fn test_keyboard_definition_asset() {
        let mut def = KeyboardDefinition::default();
        def.geometry.home_row = 0;
        def.geometry.keys.push(KeyNode::default());
        def.geometry.prime_slots.push(KeyIndex(0));
        assert_eq!(KeyboardDefinition::category(), AssetCategory::Keyboard);
        assert!(def.post_load().is_ok());
    }

    #[test]
    fn test_keyboard_definition_parse() {
        // Valid Native JSON
        let native_json = r#"{"geometry": {"keys": [{"x":0, "y":0, "hand":0, "finger":1, "row":0}], "prime_slots": [0], "med_slots": [], "low_slots": [], "home_row": 0}}"#;
        assert!(KeyboardDefinition::parse(native_json, None).is_ok());

        // Valid KLE (Simple list of arrays)
        let kle_json = r#"[["A"]]"#;
        assert!(KeyboardDefinition::parse(kle_json, Some("MyBoard")).is_ok());

        // Invalid JSON
        assert!(KeyboardDefinition::parse("not json", None).is_err());
    }

    #[test]
    fn test_keyboard_geometry_validation_extended() {
        // Too many keys
        let geom = KeyboardGeometry {
            keys: vec![KeyNode::default(); MAX_KEYBOARD_KEYS + 1],
            home_row: 0,
            ..Default::default()
        };
        assert!(geom.validate().is_err());

        // Prime and Low overlap
        let geom = KeyboardGeometry {
            keys: vec![KeyNode::default(); 2],
            prime_slots: vec![KeyIndex(0)],
            low_slots: vec![KeyIndex(0)],
            med_slots: vec![KeyIndex(1)],
            home_row: 0,
            ..Default::default()
        };
        assert!(geom.validate().is_err());

        // Med and Low overlap
        let geom = KeyboardGeometry {
            keys: vec![KeyNode::default(); 2],
            prime_slots: vec![KeyIndex(1)],
            med_slots: vec![KeyIndex(0)],
            low_slots: vec![KeyIndex(0)],
            home_row: 0,
            ..Default::default()
        };
        assert!(geom.validate().is_err());

        // Invalid finger index (using new_unchecked to bypass safety)
        let geom = KeyboardGeometry {
            keys: vec![
                KeyNode {
                    finger: FingerIndex::new_unchecked(10),
                    ..Default::default()
                },
            ],
            prime_slots: vec![KeyIndex(0)],
            home_row: 0,
            ..Default::default()
        };
        assert!(geom.validate().is_err());

        assert_eq!(geom.key_count(), 1);
    }

    #[test]
    fn test_key_node_defaults() {
        let node = KeyNode::default();
        assert_eq!(node.w, 1.0);
        assert_eq!(node.h, 1.0);
        assert!(!node.is_home);
    }
}

#[cfg(test)]
mod fuzz {
    use super::*;
    use proptest::prelude::*;

    fn geometry_strategy() -> impl Strategy<Value = KeyboardGeometry> {
        proptest::collection::vec((any::<f32>(), any::<f32>(), 0u8..10, 0u8..10), 0..300).prop_map(
            |keys| {
                let nodes = keys
                    .into_iter()
                    .map(|(w, h, hand, finger)| KeyNode {
                        w,
                        h,
                        hand: HandIndex(hand.min(1)),
                        finger: FingerIndex(finger.min(4)),
                        ..Default::default()
                    })
                    .collect();
                KeyboardGeometry {
                    keys: nodes,
                    ..Default::default()
                }
            },
        )
    }

    proptest! {
        #[test]
        fn fuzz_geometry_validation(g in geometry_strategy()) {
            let _ = g.validate();
        }
    }
}