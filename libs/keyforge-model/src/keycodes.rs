// Copyright (c) 2025 KeyForge Contributors
//
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
    name_to_code: HashMap<String, KeyCode>,
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
        // NORMALIZE: Force all Uppercase ASCII codes (65-90) to Lowercase (97-122)
        // This aligns the internal representation with standard text corpora.
        for def in &mut definitions {
            let val = def.code.0;
            if (65..=90).contains(&val) {
                def.code = KeyCode(val + 32);
            }
        }

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
