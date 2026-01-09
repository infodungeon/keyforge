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

use crate::Exporter;
use anyhow::Result;
use crate::util::{self, ModFormat};
use keyforge_adapter::parsing::{parse_key, KeyAction};

/// An exporter for the QMK (Quantum Mechanical Keyboard) firmware.
///
/// This generates a standard C keymap file compatible with `keymap.c`.
pub struct QmkExporter;

const MAX_OUTPUT_SIZE: usize = 4 * 1024 * 1024; // 4MB Limit
const MAX_KEYS: usize = 512; // Support up to 512 keys (large orthos/macros)

impl Exporter for QmkExporter {
    fn generate(&self, layout_name: &str, layers: &[Vec<String>]) -> Result<String> {
        let total_keys: usize = layers.iter().map(|l| l.len()).sum();
        if total_keys > MAX_KEYS {
            return Err(anyhow::anyhow!(
                "Too many keys for QMK export (Limit: {})",
                MAX_KEYS
            ));
        }

        let mut out = String::with_capacity(4096 * layers.len());
        // Sanitize layout name for C identifier
        let _safe_name = util::sanitize_c(&layout_name.replace(" ", "_").to_uppercase());

        out.push_str(&format!("// KeyForge QMK Export: {}\n", layout_name));
        out.push_str(&format!(
            "// Generated at: {}\n\n",
            chrono::Local::now().to_rfc3339()
        ));

        out.push_str("#include QMK_KEYBOARD_H\n\n");
        out.push_str("const uint16_t PROGMEM keymaps[][MATRIX_ROWS][MATRIX_COLS] = {\n");

        for (l_idx, keys) in layers.iter().enumerate() {
            out.push_str(&format!("  [{}] = LAYOUT(\n", l_idx));
            out.push_str("    ");

            let mut line_len = 0;
            for (i, key_str) in keys.iter().enumerate() {
                // Safety Check
                if out.len() > MAX_OUTPUT_SIZE {
                    return Err(anyhow::anyhow!("Output size limit exceeded"));
                }

                let action = parse_key(key_str);
                let code = match action {
                    KeyAction::Simple(s) => util::sanitize_c(&s),
                    KeyAction::Transparent => "_______".to_string(),
                    KeyAction::NoOp => "XXXXXXX".to_string(),
                    KeyAction::LayerMomentary(l) => format!("MO({})", l),
                    KeyAction::LayerToggle(l) => format!("TG({})", l),
                    KeyAction::LayerOn(l) => format!("TO({})", l),
                    KeyAction::ModTap { mod_name, key } => {
                        // Recursive sanitization is tricky without a full `to_qmk_code` helper.
                        // Let's implement a quick match for the inner key.
                        let key_str = match *key {
                            KeyAction::Simple(s) => util::sanitize_c(&s), 
                            KeyAction::Raw(s) => util::sanitize_c(&s),
                            _ => "TRNS".to_string(), // Fallback
                        };
                        format!("{}_T({})", util::sanitize_c(&mod_name), key_str)
                    }
                    KeyAction::LayerTap { layer, key } => {
                        let key_str = match *key {
                            KeyAction::Simple(s) => util::sanitize_c(&s),
                            KeyAction::Raw(s) => util::sanitize_c(&s),
                            _ => "TRNS".to_string(),
                        };
                        format!("LT({}, {})", layer, key_str)
                    },
                    KeyAction::StickyMod(m) => {
                        let qmk_mod = util::map_modifier(&m, ModFormat::Qmk);
                        format!("OSM({})", qmk_mod)
                    }
                    KeyAction::CapsWord => "CAPS_WORD".to_string(),
                    KeyAction::Raw(s) => util::sanitize_c(&s),
                };

                out.push_str(&code);

                if i < keys.len() - 1 {
                    out.push_str(", ");
                }

                line_len += 1;
                if line_len >= 12 {
                    out.push_str("\n    ");
                    line_len = 0;
                }
            }

            out.push_str("\n  ),\n");
        }

        out.push_str("};\n");
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qmk_generate_multi_layer() {
        let exporter = QmkExporter;
        let layers = vec![
            vec!["A".to_string(), "B".to_string()],
            vec!["TRNS".to_string(), "NO".to_string()],
        ];
        let result = exporter.generate("Test Layout", &layers).unwrap();
        
        assert!(result.contains("keymaps[][MATRIX_ROWS][MATRIX_COLS]"));
        assert!(result.contains("[0] = LAYOUT("));
        assert!(result.contains("[1] = LAYOUT("));
        assert!(result.contains("_______"));
        assert!(result.contains("XXXXXXX"));
    }
}
