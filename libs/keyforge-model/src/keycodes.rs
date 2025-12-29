use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::KeyCode;

// KeyCode is now defined in crate::types as a newtype struct

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeycodeDefinition {
    pub code: KeyCode,
    pub id: String,
    pub label: String,
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KeycodeRegistry {
    pub definitions: Vec<KeycodeDefinition>,
    name_to_code: HashMap<String, KeyCode>,
    code_to_label: HashMap<KeyCode, String>,
}

impl KeycodeRegistry {
    pub fn new(definitions: Vec<KeycodeDefinition>) -> Self {
        let mut reg = Self {
            definitions,
            name_to_code: HashMap::new(),
            code_to_label: HashMap::new(),
        };
        reg.rebuild_maps();
        reg
    }

    pub fn new_with_defaults() -> Self {
        let defs = vec![
            KeycodeDefinition {
                code: KeyCode(0),
                id: "KC_NO".into(),
                label: "".into(),
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

    pub fn get_code(&self, name: &str) -> Option<KeyCode> {
        self.name_to_code.get(&name.to_uppercase()).copied()
    }

    pub fn get_label(&self, code: KeyCode) -> String {
        self.code_to_label.get(&code).cloned().unwrap_or_else(|| format!("[{}]", code))
    }
}
