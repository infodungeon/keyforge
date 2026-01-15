// libs/keyforge-model/src/keycodes.rs

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


//! Key code definitions and registry.
//!
//! This module defines how logical key codes (like 'A' or 'Shift') are represented,
//! named, and mapped to display labels.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

use crate::validator::Validator;
use crate::types::KeyCode;

/// Definition of a logical key code (e.g., "KC_A").
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct KeycodeDefinition {
    /// The numeric code.
    pub code: KeyCode,
    /// The canonical ID (e.g., "KC_A").
    pub id: String,
    /// The display label (e.g., "A").
    pub label: String,
    /// Alternative names (e.g., ["KC_1", "1"]).
    pub aliases: Vec<String>,
}

impl Validator for KeycodeDefinition {
    fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() {
            return Err(format!("Keycode {} has empty ID", self.code));
        }

        if self.label.is_empty() {
            return Err(format!("Keycode {} ({}) has empty label", self.code, self.id));
        }
        Ok(())
    }
}

/// Registry for looking up key codes by name or ID.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct KeycodeRegistry {
    /// List of all definitions.
    pub definitions: Vec<KeycodeDefinition>,
    #[serde(skip)]
    name_to_code: HashMap<String, KeyCode>,
    #[serde(skip)]
    code_to_label: HashMap<KeyCode, String>,
}

impl Validator for KeycodeRegistry {
    fn validate(&self) -> Result<(), String> {
        let mut seen_codes = std::collections::HashSet::new();
        let mut seen_ids = std::collections::HashSet::new();

        for def in &self.definitions {
            def.validate()?;
            if !seen_codes.insert(def.code) {
                return Err(format!("Duplicate KeyCode: {}", def.code));
            }
            if !seen_ids.insert(def.id.to_uppercase()) {
                return Err(format!("Duplicate Key ID: {}", def.id));
            }
        }
        Ok(())
    }
}

impl KeycodeRegistry {
    /// Creates a new registry from a list of definitions.
    pub fn new(mut definitions: Vec<KeycodeDefinition>) -> Self {
        for def in &mut definitions {
            // 0. QMK to ASCII Remapping (Heuristic fix for physics scoring)
            if let Some(ascii) = qmk_to_ascii(def.code.0) {
                def.code = KeyCode(ascii);
            }

            // 1. NORMALIZE: Force all Uppercase ASCII Alphas (A-Z) to Lowercase (a-z)
            // This ensures consistence between corpus text (which is lowercased for heatmaps) 
            // and key definitions.
            let val = def.code.0;
            if (b'A'..=b'Z').contains(&(val as u8)) {
                def.code = KeyCode((val as u8).to_ascii_lowercase() as u16);
            }
        }

        // 2. DEDUPLICATE: If multiple definitions now have the same code (e.g. 'A' and 'a'), 
        // keep the one with the lowercase code or the first one encountered.
        let mut seen = std::collections::HashSet::new();
        definitions.retain(|def| seen.insert(def.code));

        let mut reg = Self {
            definitions,
            name_to_code: HashMap::new(),
            code_to_label: HashMap::new(),
        };
        reg.rebuild_maps();
        reg
    }

    /// Creates a registry with minimal defaults (KC_NO, KC_TRNS).
    pub fn new_with_defaults() -> Self {
        let defs = vec![
            KeycodeDefinition {
                code: KeyCode(0),
                id: "KC_NO".into(),
                label: " ".into(),
                aliases: vec!["XXXXXXX".into()],
            },
            KeycodeDefinition {
                code: KeyCode(1),
                id: "KC_TRANSPARENT".into(),
                label: "▽".into(),
                aliases: vec!["KC_TRNS".into(), "_______".into()],
            },
        ];
        Self::new(defs)
    }

    /// Rebuilds the internal lookup maps.
    pub fn rebuild_maps(&mut self) {
        self.name_to_code.clear();
        self.code_to_label.clear();
        for def in &self.definitions {
            self.code_to_label.insert(def.code, def.label.clone());
            self.name_to_code.insert(def.id.to_uppercase(), def.code);
            self.name_to_code.insert(def.label.to_uppercase(), def.code);
            for alias in &def.aliases {
                self.name_to_code.insert(alias.to_uppercase(), def.code);
            }
        }
    }

    /// Looks up a KeyCode by name (case-insensitive).
    pub fn get_code(&self, name: &str) -> Option<KeyCode> {
        self.name_to_code.get(&name.to_uppercase()).copied()
    }

    /// Gets the display label for a KeyCode.
    pub fn get_label(&self, code: KeyCode) -> String {
        self.code_to_label.get(&code).cloned().unwrap_or_else(|| format!("[{}]", code))
    }
}

fn qmk_to_ascii(qmk: u16) -> Option<u16> {
    match qmk {
        4..=29 => Some(qmk - 4 + 97),   // a-z
        30..=38 => Some(qmk - 30 + 49), // 1-9
        39 => Some(48),                 // 0
        40 => Some(10),                 // Enter
        41 => Some(27),                 // Escape
        42 => Some(8),                  // Backspace
        43 => Some(9),                  // Tab
        44 => Some(32),                 // Space
        45 => Some(45),                 // -
        46 => Some(61),                 // =
        47 => Some(91),                 // [
        48 => Some(93),                 // ]
        49 => Some(92),                 // \
        51 => Some(59),                 // ;
        52 => Some(39),                 // '
        53 => Some(96),                 // `
        54 => Some(44),                 // ,
        55 => Some(46),                 // .
        56 => Some(47),                 // /
        _ => None,
    }
}
