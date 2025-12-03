use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeycodeDefinition {
    pub code: u16, // CHANGED: u8 -> u16
    pub id: String,
    pub label: String,
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeycodeRegistry {
    #[serde(skip)]
    name_to_code: HashMap<String, u16>, // CHANGED: u8 -> u16

    #[serde(skip)]
    code_to_label: HashMap<u16, String>, // CHANGED: u8 -> u16

    pub definitions: Vec<KeycodeDefinition>,
}

impl KeycodeRegistry {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read keycodes file: {}", e))?;

        let definitions: Vec<KeycodeDefinition> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse keycodes JSON: {}", e))?;

        let mut reg = Self {
            definitions,
            name_to_code: HashMap::new(),
            code_to_label: HashMap::new(),
        };

        reg.rebuild_maps();
        Ok(reg)
    }

    pub fn new_with_defaults() -> Self {
        let mut reg = Self::default();

        // 1. Add ASCII Printable (0-127 are safe in u16)
        for b in 32..=126 {
            let char_str = String::from_utf8(vec![b as u8]).unwrap();
            reg.definitions.push(KeycodeDefinition {
                code: b,
                id: format!("KC_{}", char_str.to_uppercase()),
                label: char_str.clone(),
                aliases: vec![char_str],
            });
        }

        // 2. Add Essential Control Codes
        let defaults = vec![
            (9, "KC_TAB", "Tab", vec!["TAB"]),
            (10, "KC_ENT", "Enter", vec!["ENT", "ENTER", "RET", "RETURN"]),
            (27, "KC_ESC", "Esc", vec!["ESC", "ESCAPE"]),
            (8, "KC_BSPC", "Bksp", vec!["BSP", "BACKSPACE"]),
            (32, "KC_SPC", "Space", vec!["SPC", "SPACE"]),
            (127, "KC_DEL", "Del", vec!["DEL", "DELETE"]),
            // Modifiers - QMK ranges often start at 0x00E0 (224) but we use custom here
            (128, "KC_LCTL", "Ctrl", vec!["LCTRL", "CTRL", "CTL"]),
            (129, "KC_RCTL", "RCtl", vec!["RCTRL"]),
            (130, "KC_LSFT", "Shift", vec!["LSHIFT", "SHIFT", "SFT"]),
            (131, "KC_RSFT", "RSft", vec!["RSHIFT"]),
            (132, "KC_LALT", "Alt", vec!["LALT", "ALT", "OPT"]),
            (133, "KC_RALT", "RAlt", vec!["RALT", "ALTGR"]),
            (134, "KC_LGUI", "Gui", vec!["LGUI", "GUI", "CMD", "WIN"]),
            // Layers
            (136, "MO(1)", "L1", vec!["MO1", "LOWER"]),
            (137, "MO(2)", "L2", vec!["MO2", "RAISE"]),
            // Special
            (1, "KC_TRNS", "▽", vec!["TRNS", "_", "_______"]),
            (0, "KC_NO", "", vec!["NO", "XXX", "XXXXXXX"]),
        ];

        for (code, id, label, aliases) in defaults {
            reg.definitions.push(KeycodeDefinition {
                code,
                id: id.to_string(),
                label: label.to_string(),
                aliases: aliases.iter().map(|s| s.to_string()).collect(),
            });
        }

        reg.rebuild_maps();
        reg
    }

    fn rebuild_maps(&mut self) {
        self.name_to_code.clear();
        self.code_to_label.clear();

        for def in &self.definitions {
            self.code_to_label.insert(def.code, def.label.clone());
            self.name_to_code.insert(def.id.to_uppercase(), def.code);

            for alias in &def.aliases {
                self.name_to_code.insert(alias.to_uppercase(), def.code);
            }
        }

        // Inject raw ASCII
        for b in 32..=126u16 {
            use std::collections::hash_map::Entry;
            if let Entry::Vacant(e) = self.code_to_label.entry(b) {
                let s = String::from_utf8(vec![b as u8]).unwrap();
                e.insert(s.clone());
                self.name_to_code.insert(s.to_uppercase(), b);
            }
        }
    }

    pub fn get_code(&self, token: &str) -> Option<u16> {
        self.name_to_code.get(&token.to_uppercase()).copied()
    }

    pub fn get_label(&self, code: u16) -> String {
        self.code_to_label.get(&code).cloned().unwrap_or_else(|| {
            if code >= 256 {
                format!("[#{}]", code) // Dynamic/Unknown range
            } else {
                String::from_utf8(vec![code as u8]).unwrap_or("?".to_string())
            }
        })
    }
}
