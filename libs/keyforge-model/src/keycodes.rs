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
use std::fmt;
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

use crate::asset::{Asset, AssetCategory};
use crate::error::ForgeError;
use crate::types::KeyCode;
use crate::validator::Validator;

use crate::constants::{DEFAULT_NO_OP, DEFAULT_TRANSPARENT};

/// Definition of a logical key code (e.g., "`KC_A`").
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct KeycodeDefinition {
    /// The numeric code.
    pub code: KeyCode,
    /// The canonical ID (e.g., "`KC_A`").
    pub id: String,
    /// The display label (e.g., "A").
    pub label: String,
    /// Alternative names (e.g., [`KC_1`, `1`]).
    pub aliases: Vec<String>,
}

impl fmt::Display for KeycodeDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {} ({})", self.id, self.code, self.label)
    }
}

impl Validator for KeycodeDefinition {
    fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() {
            return Err(format!("Keycode {} has empty ID", self.code));
        }

        if self.label.is_empty() {
            return Err(format!(
                "Keycode {} ({}) has empty label",
                self.code, self.id
            ));
        }
        Ok(())
    }
}

/// Registry for looking up key codes by name or ID.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(from = "Vec<KeycodeDefinition>", into = "Vec<KeycodeDefinition>")]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct KeycodeRegistry {
    /// List of all definitions.
    pub definitions: Vec<KeycodeDefinition>,
    #[serde(skip)]
    name_to_code: HashMap<String, KeyCode>,
    #[serde(skip)]
    code_to_label: HashMap<KeyCode, String>,
}

impl From<Vec<KeycodeDefinition>> for KeycodeRegistry {
    fn from(defs: Vec<KeycodeDefinition>) -> Self {
        Self::new(defs)
    }
}

impl From<KeycodeRegistry> for Vec<KeycodeDefinition> {
    fn from(reg: KeycodeRegistry) -> Self {
        reg.definitions
    }
}

impl Asset for KeycodeRegistry {
    fn category() -> AssetCategory {
        AssetCategory::Keycodes
    }

    fn post_load(&mut self) -> Result<(), ForgeError> {
        self.rebuild_maps();
        self.validate().map_err(ForgeError::InvalidData)
    }
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
    #[must_use]
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
            #[allow(clippy::cast_possible_truncation)]
            if (val as u8).is_ascii_uppercase() {
                def.code = KeyCode(u16::from((val as u8).to_ascii_lowercase()));
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

    /// Creates a registry with minimal defaults (`KC_NO`, `KC_TRNS`).
    #[must_use]
    pub fn new_with_defaults() -> Self {
        let defs = vec![
            KeycodeDefinition {
                code: KeyCode(0),
                id: "KC_NO".into(),
                label: " ".into(),
                aliases: vec![DEFAULT_NO_OP.into()],
            },
            KeycodeDefinition {
                code: KeyCode(1),
                id: "KC_TRANSPARENT".into(),
                label: "▽".into(),
                aliases: vec!["KC_TRNS".into(), DEFAULT_TRANSPARENT.into()],
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

    /// Looks up a `KeyCode` by name (case-insensitive).
    #[must_use]
    pub fn get_code(&self, name: &str) -> Option<KeyCode> {
        self.name_to_code.get(&name.to_uppercase()).copied()
    }

    /// Gets the display label for a `KeyCode`.
    #[must_use]
    pub fn get_label(&self, code: KeyCode) -> String {
        self.code_to_label
            .get(&code)
            .cloned()
            .unwrap_or_else(|| format!("[{code}]"))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keycode_definition_validation() {
        let valid = KeycodeDefinition {
            code: KeyCode(10),
            id: "KC_A".into(),
            label: "A".into(),
            aliases: vec![],
        };
        assert!(valid.validate().is_ok());

        let empty_id = KeycodeDefinition {
            code: KeyCode(10),
            id: " ".into(),
            label: "A".into(),
            aliases: vec![],
        };
        assert!(empty_id.validate().is_err());

        let empty_label = KeycodeDefinition {
            code: KeyCode(10),
            id: "KC_A".into(),
            label: String::new(),
            aliases: vec![],
        };
                assert!(empty_label.validate().is_err());
            }
        
            #[test]
            fn test_keycode_definition_display() {
                let def = KeycodeDefinition {
                    code: KeyCode(10),
                    id: "KC_A".into(),
                    label: "A".into(),
                    aliases: vec![],
                };
                assert_eq!(format!("{def}"), "KC_A: 10 (A)");
            }
        
            #[test]
            fn test_keycode_registry_normalization() {

            let defs = vec![

                KeycodeDefinition { code: KeyCode(65), id: "A".into(), label: "A".into(), aliases: vec![] }, // 'A' -> 'a'

                KeycodeDefinition { code: KeyCode(97), id: "a_lower".into(), label: "a".into(), aliases: vec![] }, // Duplicate 'a'

                KeycodeDefinition { code: KeyCode(4), id: "KC_A_QMK".into(), label: "A".into(), aliases: vec![] }, // QMK 4 -> ASCII 97

            ];

            let reg = KeycodeRegistry::new(defs);

            // Normalized and deduplicated: should only have 1 definition for code 97

            assert_eq!(reg.definitions.len(), 1);

            assert_eq!(reg.definitions[0].code.0, 97);

        }

    

        #[test]

        fn test_keycode_registry_lookups() {

            let reg = KeycodeRegistry::new_with_defaults();

            assert_eq!(reg.get_code("KC_NO"), Some(KeyCode(0)));

            assert_eq!(reg.get_code("kc_no"), Some(KeyCode(0))); // Case insensitive

            assert_eq!(reg.get_code("NONEXISTENT"), None);

            

            assert_eq!(reg.get_label(KeyCode(0)), " ");

            assert_eq!(reg.get_label(KeyCode(999)), "[999]"); // Fallback

        }

    

        #[test]

        fn test_keycode_asset_and_conversions() {

            let reg = KeycodeRegistry::new_with_defaults();

            assert_eq!(KeycodeRegistry::category(), AssetCategory::Keycodes);

            

            let mut reg_clone = reg.clone();

            assert!(reg_clone.post_load().is_ok());

    

            let defs: Vec<KeycodeDefinition> = reg.clone().into();

            assert_eq!(defs.len(), reg.definitions.len());

            

            let reg_from: KeycodeRegistry = defs.into();

            assert_eq!(reg_from.definitions.len(), reg.definitions.len());

        }

    

            #[test]
            fn test_qmk_to_ascii_mapping_exhaustive() {
                // Test remaining branches for full coverage
                assert_eq!(qmk_to_ascii(39), Some(48)); // 0
                assert_eq!(qmk_to_ascii(40), Some(10)); // Enter
                assert_eq!(qmk_to_ascii(41), Some(27)); // Escape
                assert_eq!(qmk_to_ascii(42), Some(8));  // Backspace
                        assert_eq!(qmk_to_ascii(43), Some(9));  // Tab
                        assert_eq!(qmk_to_ascii(44), Some(32)); // Space
                        assert_eq!(qmk_to_ascii(45), Some(45)); // -
                        assert_eq!(qmk_to_ascii(46), Some(61)); // =
                        assert_eq!(qmk_to_ascii(47), Some(91)); // [
                        assert_eq!(qmk_to_ascii(48), Some(93)); // ]
                        assert_eq!(qmk_to_ascii(49), Some(92)); // \
                        assert_eq!(qmk_to_ascii(51), Some(59)); // ;
                        assert_eq!(qmk_to_ascii(52), Some(39)); // '
                        assert_eq!(qmk_to_ascii(53), Some(96)); // `
                        assert_eq!(qmk_to_ascii(54), Some(44)); // ,
                        assert_eq!(qmk_to_ascii(55), Some(46)); // .
                        assert_eq!(qmk_to_ascii(56), Some(47)); // /
                        assert_eq!(qmk_to_ascii(100), None);
                    }        
            #[test]
            fn test_keycode_registry_validation_duplicates() {
                let mut reg = KeycodeRegistry::default();
                reg.definitions.push(KeycodeDefinition { code: KeyCode(10), id: "A".into(), label: "A".into(), aliases: vec![] });
                reg.definitions.push(KeycodeDefinition { code: KeyCode(11), id: "a".into(), label: "B".into(), aliases: vec![] });
                assert!(reg.validate().is_err(), "Should fail on duplicate ID (case-insensitive)");
        
                let mut reg = KeycodeRegistry::default();
                reg.definitions.push(KeycodeDefinition { code: KeyCode(10), id: "A".into(), label: "A".into(), aliases: vec![] });
                reg.definitions.push(KeycodeDefinition { code: KeyCode(10), id: "B".into(), label: "B".into(), aliases: vec![] });
                assert!(reg.validate().is_err(), "Should fail on duplicate Code");
            }
        }
