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
use crate::config::weights::constants::{
    DEFAULT_THRESHOLD_SCISSOR_ROW_DIFF, DEFAULT_THRESHOLD_SFB_LONG_ROW_DIFF,
};
use crate::constants::{MAX_KEYBOARD_KEYS, MAX_KEYBOARD_NAME_LEN};
use crate::error::ForgeError;
use crate::types::{ColIndex, FingerIndex, HandIndex, KeyIndex, RowIndex, SpatialUnit};
use crate::validator::Validator;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};

/// Metadata describing a keyboard definition.
#[derive(Debug, Clone, Default)]
pub struct KeyboardMeta {
    /// Display name of the keyboard.
    pub name: String,
    /// Author of the definition.
    pub author: String,
    /// Version string.
    pub version: String,
    /// Additional notes or description.
    pub notes: String,
    /// Type of keyboard (e.g., "split", "ortho").
    pub kb_type: String,
}

/// Complete definition of a keyboard, including metadata and geometry.
#[derive(Debug, Clone, Default)]
pub struct KeyboardDefinition {
    /// Metadata about the keyboard.
    pub meta: KeyboardMeta,
    /// Physical geometry of the keys.
    pub geometry: KeyboardGeometry,
    /// Pre-defined layouts available for this keyboard.
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
#[derive(Debug, Clone, PartialEq)]
pub struct KeyNode {
    /// Unique index of the key in the layout.
    pub index: KeyIndex,
    /// Descriptive label for the key (e.g., "K01", "Thumb").
    pub label: String,
    /// X-coordinate of the key center in keyboard units.
    pub x: SpatialUnit,
    /// Y-coordinate of the key center in keyboard units.
    pub y: SpatialUnit,
    /// Width of the key in keyboard units (default 1.0).
    pub w: f32,
    /// Height of the key in keyboard units (default 1.0).
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
    pub is_home: bool,
    /// Whether this key requires a stretch to reach.
    pub is_stretch: bool,
    /// Rotation angle in degrees.
    pub r: f32,
    /// Rotation center X.
    pub rx: SpatialUnit,
    /// Rotation center Y.
    pub ry: SpatialUnit,
}

impl Default for KeyNode {
    fn default() -> Self {
        Self {
            index: KeyIndex::new(0),
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

impl KeyNode {
    /// Returns the X-coordinate.
    #[must_use]
    pub const fn x(&self) -> SpatialUnit {
        self.x
    }

    /// Returns the Y-coordinate.
    #[must_use]
    pub const fn y(&self) -> SpatialUnit {
        self.y
    }

    /// Returns the width.
    #[must_use]
    pub const fn w(&self) -> f32 {
        self.w
    }

    /// Returns the height.
    #[must_use]
    pub const fn h(&self) -> f32 {
        self.h
    }

    /// Returns the rotation center X.
    #[must_use]
    pub const fn rx(&self) -> SpatialUnit {
        self.rx
    }

    /// Returns the rotation center Y.
    #[must_use]
    pub const fn ry(&self) -> SpatialUnit {
        self.ry
    }

    /// Returns the point (X, Y).
    #[must_use]
    pub fn point(&self) -> crate::types::Point {
        crate::types::Point::new(self.x, self.y)
    }

    /// Returns a builder for `KeyNode`.
    #[must_use]
    pub fn builder() -> KeyNodeBuilder {
        KeyNodeBuilder::default()
    }
}

/// Builder for `KeyNode`.
#[derive(Debug, Clone, Default)]
pub struct KeyNodeBuilder {
    index: KeyIndex,
    label: String,
    x: SpatialUnit,
    y: SpatialUnit,
    w: Option<f32>,
    h: Option<f32>,
    hand: HandIndex,
    finger: FingerIndex,
    row: RowIndex,
    col: ColIndex,
    is_home: bool,
    is_stretch: bool,
    r: f32,
    rx: SpatialUnit,
    ry: SpatialUnit,
}

impl KeyNodeBuilder {
    /// Sets the key index.
    #[must_use]
    pub fn index(mut self, index: KeyIndex) -> Self {
        self.index = index;
        self
    }
    /// Sets the label.
    #[must_use]
    pub fn label(mut self, label: String) -> Self {
        self.label = label;
        self
    }
    /// Sets the X coordinate.
    #[must_use]
    pub fn x(mut self, x: SpatialUnit) -> Self {
        self.x = x;
        self
    }
    /// Sets the Y coordinate.
    #[must_use]
    pub fn y(mut self, y: SpatialUnit) -> Self {
        self.y = y;
        self
    }
    /// Sets the width.
    #[must_use]
    pub fn w(mut self, w: f32) -> Self {
        self.w = Some(w);
        self
    }
    /// Sets the height.
    #[must_use]
    pub fn h(mut self, h: f32) -> Self {
        self.h = Some(h);
        self
    }
    /// Sets the hand.
    #[must_use]
    pub fn hand(mut self, hand: HandIndex) -> Self {
        self.hand = hand;
        self
    }
    /// Sets the finger.
    #[must_use]
    pub fn finger(mut self, finger: FingerIndex) -> Self {
        self.finger = finger;
        self
    }
    /// Sets the row.
    #[must_use]
    pub fn row(mut self, row: RowIndex) -> Self {
        self.row = row;
        self
    }
    /// Sets the column.
    #[must_use]
    pub fn col(mut self, col: ColIndex) -> Self {
        self.col = col;
        self
    }
    /// Sets home row status.
    #[must_use]
    pub fn is_home(mut self, is_home: bool) -> Self {
        self.is_home = is_home;
        self
    }
    /// Sets stretch status.
    #[must_use]
    pub fn is_stretch(mut self, is_stretch: bool) -> Self {
        self.is_stretch = is_stretch;
        self
    }
    /// Sets rotation.
    #[must_use]
    pub fn r(mut self, r: f32) -> Self {
        self.r = r;
        self
    }
    /// Sets rotation center X.
    #[must_use]
    pub fn rx(mut self, rx: SpatialUnit) -> Self {
        self.rx = rx;
        self
    }
    /// Sets rotation center Y.
    #[must_use]
    pub fn ry(mut self, ry: SpatialUnit) -> Self {
        self.ry = ry;
        self
    }

    /// Builds the `KeyNode`.
    #[must_use]
    pub fn build(self) -> KeyNode {
        KeyNode {
            index: self.index,
            label: self.label,
            x: self.x,
            y: self.y,
            w: self.w.unwrap_or(1.0),
            h: self.h.unwrap_or(1.0),
            hand: self.hand,
            finger: self.finger,
            row: self.row,
            col: self.col,
            is_home: self.is_home,
            is_stretch: self.is_stretch,
            r: self.r,
            rx: self.rx,
            ry: self.ry,
        }
    }
}

/// Collection of keys and slot definitions defining the keyboard geometry.
#[derive(Debug, Clone)]
pub struct KeyboardGeometry {
    /// List of physical keys.
    pub keys: Vec<KeyNode>,
    /// Indices of keys that are "prime" (highest efficiency).
    pub prime_slots: Vec<KeyIndex>,
    /// Indices of keys that are "med" (medium efficiency).
    pub med_slots: Vec<KeyIndex>,
    /// Indices of keys that are "low" (lowest efficiency).
    pub low_slots: Vec<KeyIndex>,
    /// Logical index of the home row.
    pub home_row: RowIndex,
    /// Row difference threshold for "long" SFBs.
    pub threshold_sfb_long_row_diff: i8,
    /// Row difference threshold for scissors.
    pub threshold_scissor_row_diff: i8,
}

impl Default for KeyboardGeometry {
    fn default() -> Self {
        Self {
            keys: Vec::new(),
            prime_slots: Vec::new(),
            med_slots: Vec::new(),
            low_slots: Vec::new(),
            home_row: RowIndex::new(0),
            threshold_sfb_long_row_diff: DEFAULT_THRESHOLD_SFB_LONG_ROW_DIFF,
            threshold_scissor_row_diff: DEFAULT_THRESHOLD_SCISSOR_ROW_DIFF,
        }
    }
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
            threshold_sfb_long_row_diff: DEFAULT_THRESHOLD_SFB_LONG_ROW_DIFF,
            threshold_scissor_row_diff: DEFAULT_THRESHOLD_SCISSOR_ROW_DIFF,
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

    /// Returns the SFB long row difference threshold.
    #[must_use]
    pub fn threshold_sfb_long_row_diff(&self) -> i8 {
        self.threshold_sfb_long_row_diff
    }

    /// Returns the scissor row difference threshold.
    #[must_use]
    pub fn threshold_scissor_row_diff(&self) -> i8 {
        self.threshold_scissor_row_diff
    }

    /// Sets the thresholds for the keyboard geometry.
    #[must_use]
    pub fn with_thresholds(mut self, sfb_long: i8, scissor: i8) -> Self {
        self.threshold_sfb_long_row_diff = sfb_long;
        self.threshold_scissor_row_diff = scissor;
        self
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

    /// Generates a deterministic hash of the keyboard geometry.
    #[must_use]
    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        for key in &self.keys {
            hasher.update(key.index.raw().to_le_bytes());
            // Use bit_cast to ensure consistent representation of floats
            hasher.update(key.x.raw().to_le_bytes());
            hasher.update(key.y.raw().to_le_bytes());
            hasher.update(key.w.to_bits().to_le_bytes());
            hasher.update(key.h.to_bits().to_le_bytes());
            hasher.update(key.hand.as_u8().to_le_bytes());
            hasher.update(key.finger.as_u8().to_le_bytes());
            hasher.update(key.row.raw().to_le_bytes());
            hasher.update(key.col.raw().to_le_bytes());
            hasher.update([u8::from(key.is_home)]);
            hasher.update([u8::from(key.is_stretch)]);
            hasher.update(key.r.to_bits().to_le_bytes());
            hasher.update(key.rx.raw().to_le_bytes());
            hasher.update(key.ry.raw().to_le_bytes());
        }
        for idx in &self.prime_slots {
            hasher.update(idx.raw().to_le_bytes());
        }
        for idx in &self.med_slots {
            hasher.update(idx.raw().to_le_bytes());
        }
        for idx in &self.low_slots {
            hasher.update(idx.raw().to_le_bytes());
        }
        hasher.update(self.home_row.raw().to_le_bytes());
        hex::encode(hasher.finalize())
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
            if usize::from(idx.raw()) >= max_idx {
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
            .map(|i| {
                let u16_val = u16::try_from(i).unwrap_or(u16::MAX);
                let f32_val = f32::from(u16_val);
                let i8_val = i8::try_from(i % 128).unwrap_or(0);
                KeyNode {
                    index: KeyIndex::new(u16_val),
                    label: format!("K{i:02}"),
                    x: SpatialUnit::from_f32(f32_val),
                    y: SpatialUnit::default(),
                    hand: HandIndex::new(0),
                    finger: FingerIndex::new_unchecked(0),
                    row: RowIndex::new(0),
                    col: ColIndex::new(i8_val),
                    ..Default::default()
                }
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
                prime_slots: (0..u16::try_from(keys_count).unwrap_or(0))
                    .map(KeyIndex::new)
                    .collect(),
                home_row: RowIndex::new(0),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
