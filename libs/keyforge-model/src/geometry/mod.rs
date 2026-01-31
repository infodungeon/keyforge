// libs/keyforge-model/src/geometry/mod.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You    may obtain a copy of the License at
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
use crate::types::{ColIndex, FingerIndex, HandIndex, KeyIndex, RowIndex, SpatialUnit};
use crate::validator::Validator;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use utoipa::ToSchema;

/// Keyboard Layout Editor (KLE) integration.
pub mod kle;

/// Metadata describing a keyboard definition.
#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]

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
            return Err(format!(
                "Keyboard name too long (max {MAX_KEYBOARD_NAME_LEN})"
            ));
        }
        self.geometry.validate()
    }
}

/// Represents a single physical key on the keyboard.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]

pub struct KeyNode {
    /// Zero-based index of the key in the layout.
    #[serde(default)]
    pub index: usize,
    /// Descriptive label for the key (e.g., "K01", "Thumb").
    #[serde(alias = "id")]
    pub label: String,
    /// X-coordinate of the key center in keyboard units.
    pub x: SpatialUnit,
    /// Y-coordinate of the key center in keyboard units.
    pub y: SpatialUnit,
    /// Width of the key in keyboard units (default 1.0).
    #[serde(default = "default_size")]
    pub w: f32,
    /// Height of the key in keyboard units (default 1.0).
    #[serde(default = "default_size")]
    pub h: f32,
    /// Which hand is responsible for this key.
    pub hand: HandIndex,
    /// Which finger is responsible for this key.
    pub finger: FingerIndex,
    /// Row index (logical).
    pub row: RowIndex,
    /// Column index (logical).
    pub col: ColIndex,
    /// Whether this key is part of the "home" position.
    #[serde(default)]
    pub is_home: bool,
    /// Whether this key requires a stretch to reach.
    #[serde(default)]
    pub is_stretch: bool,
    /// Rotation angle in degrees.
    #[serde(default)]
    pub r: f32,
    /// Rotation center X.
    #[serde(default)]
    pub rx: SpatialUnit,
    /// Rotation center Y.
    #[serde(default)]
    pub ry: SpatialUnit,
}

impl Default for KeyNode {
    fn default() -> Self {
        Self {
            index: 0,
            label: String::new(),
            x: SpatialUnit::default(),
            y: SpatialUnit::default(),
            w: 1.0,
            h: 1.0,
            hand: HandIndex::new(0),
            finger: FingerIndex::new_unchecked(0),
            row: RowIndex::new(0),
            col: ColIndex::new(0),
            is_home: false,
            is_stretch: false,
            r: 0.0,
            rx: SpatialUnit::default(),
            ry: SpatialUnit::default(),
        }
    }
}

fn default_size() -> f32 {
    1.0
}

/// Collection of keys and slot definitions defining the keyboard geometry.
///
/// NOTE: Uses `Vec<T>` for serialization compatibility with `utoipa` and `ts-rs`.
/// The `Keyboard` runtime structure uses `Arc<[T]>` for performance.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]

pub struct KeyboardGeometry {
    /// List of physical keys.
    pub keys: Vec<KeyNode>,
    /// Indices of keys that are "prime" (highest efficiency).
    #[serde(default)]
    pub prime_slots: Vec<KeyIndex>,
    /// Indices of keys that are "med" (medium efficiency).
    #[serde(default)]
    pub med_slots: Vec<KeyIndex>,
    /// Indices of keys that are "low" (lowest efficiency).
    #[serde(default)]
    pub low_slots: Vec<KeyIndex>,
    /// Logical index of the home row.
    #[serde(default)]
    pub home_row: RowIndex,
}

impl KeyboardGeometry {
    /// Creates a new `KeyboardGeometry`.
    #[must_use]
    pub fn new(
        keys: Vec<KeyNode>,
        prime_slots: Vec<KeyIndex>,
        med_slots: Vec<KeyIndex>,
        low_slots: Vec<KeyIndex>,
        home_row: RowIndex,
    ) -> Self {
        Self {
            keys,
            prime_slots,
            med_slots,
            low_slots,
            home_row,
        }
    }

    /// Returns a reference to the keys.
    #[must_use]
    pub fn keys(&self) -> &[KeyNode] {
        &self.keys
    }

    /// Returns the home row index.
    #[must_use]
    pub fn home_row(&self) -> RowIndex {
        self.home_row
    }

    /// Returns the prime slots.
    #[must_use]
    pub fn prime_slots(&self) -> &[KeyIndex] {
        &self.prime_slots
    }

    /// Returns the medium slots.
    #[must_use]
    pub fn med_slots(&self) -> &[KeyIndex] {
        &self.med_slots
    }

    /// Returns the low slots.
    #[must_use]
    pub fn low_slots(&self) -> &[KeyIndex] {
        &self.low_slots
    }
}

impl Validator for KeyboardGeometry {
    fn validate(&self) -> Result<(), String> {
        let has_home_keys = self.keys.iter().any(|k| k.is_home);
        let has_home_row_matches = self.keys.iter().any(|k| k.row == self.home_row);

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
            if (idx.raw() as usize) >= max_idx {
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
    /// Returns an error if the content is not valid JSON or KLE format.
    pub fn parse(content: &str, name_hint: Option<&str>) -> Result<Self, String> {
        if let Ok(def) = serde_json::from_str::<KeyboardDefinition>(content) {
            return Ok(def);
        }
        if let Ok(geom) = kle::parse_kle_json(content) {
            let name = name_hint.unwrap_or("Imported Board").to_string();
            return Ok(Self::from_geometry(geom, &name));
        }
        Err("Content is not a valid KeyForge or KLE JSON".to_string())
    }

    /// Creates a new `KeyboardDefinition` from a `KeyboardGeometry`.
    #[must_use]
    pub fn from_geometry(geometry: KeyboardGeometry, name: &str) -> Self {
        Self {
            meta: KeyboardMeta {
                name: name.to_string(),
                kb_type: "imported".to_string(),
                ..Default::default()
            },
            geometry,
            layouts: HashMap::new(),
        }
    }

    /// Generates a test keyboard definition with N keys.
    #[must_use]
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    pub fn generate_test(keys_count: usize) -> Self {
        let keys: Vec<KeyNode> = (0..keys_count)
            .map(|i| KeyNode {
                index: i,
                label: format!("K{i:02}"),
                x: SpatialUnit::from_f32(i as f32),
                y: SpatialUnit::default(),
                hand: HandIndex::new(0),
                finger: FingerIndex::new_unchecked(0),
                row: RowIndex::new(0),
                col: ColIndex::new(i as i8),
                ..Default::default()
            })
            .collect();

        Self {
            meta: KeyboardMeta {
                name: format!("Test-{keys_count}"),
                author: "system".to_string(),
                ..Default::default()
            },
            geometry: KeyboardGeometry {
                keys,
                prime_slots: (0..keys_count as u16).map(KeyIndex::new).collect(),
                home_row: RowIndex::new(0),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
