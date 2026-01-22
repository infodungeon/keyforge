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

use crate::error::{ExportError, ExportResult};
use crate::util::{self, ModFormat};
use crate::Exporter;
use keyforge_adapter::parsing::{parse_key, KeyAction};
use keyforge_model::keycodes::KeycodeRegistry;

use keyforge_model::constants::{DEFAULT_NO_OP, DEFAULT_TRANSPARENT};
use std::fmt::Write;

/// An exporter for the QMK (Quantum Mechanical Keyboard) firmware.
#[derive(Debug)]
pub struct QmkExporter;

const MAX_OUTPUT_SIZE: usize = 65536; // 4MB Limit
const MAX_KEYS: usize = 256; // Support up to 512 keys (large orthos/macros)

impl Exporter for QmkExporter {
    fn generate(
        &self,
        layout_name: &str,
        layers: &[Vec<String>],
        registry: Option<&KeycodeRegistry>,
    ) -> ExportResult<String> {
        let total_keys: usize = layers.iter().map(std::vec::Vec::len).sum();
        if total_keys > MAX_KEYS {
            return Err(ExportError::TooManyKeys(MAX_KEYS));
        }

        let mut out = String::with_capacity(4096 * layers.len());
        // Sanitize layout name for C identifier
        let _safe_name = util::sanitize_c(&layout_name.replace(' ', "_").to_uppercase());

        let _ = writeln!(out, "// KeyForge QMK Export: {layout_name}");
        let _ = write!(
            out,
            "// Generated at: {}\n\n",
            chrono::Local::now().to_rfc3339()
        );

        out.push_str("#include QMK_KEYBOARD_H\n\n");
        out.push_str("const uint16_t PROGMEM keymaps[][MATRIX_ROWS][MATRIX_COLS] = {\n");

        for (l_idx, keys) in layers.iter().enumerate() {
            let _ = writeln!(out, "  [{l_idx}] = LAYOUT(");
            out.push_str("    ");

            let mut line_len = 0;
            for (i, key_str) in keys.iter().enumerate() {
                // Safety Check
                if out.len() > MAX_OUTPUT_SIZE {
                    return Err(ExportError::OutputSizeLimitExceeded);
                }

                let action = parse_key(key_str).unwrap_or(KeyAction::Simple(key_str.clone()));
                let code = action_to_qmk(&action, registry);

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

fn action_to_qmk(action: &KeyAction, registry: Option<&KeycodeRegistry>) -> String {
    match action {
        KeyAction::Simple(s) => {
            if let Some(reg) = registry {
                // Try looking up the canonical ID if the input was a label or alias
                if let Some(code) = reg.resolve_token(s) {
                    if let Some(def) = reg.definitions.iter().find(|d| d.code == code) {
                        return util::sanitize_c(&def.id);
                    }
                }
            }
            util::sanitize_c(s)
        }
        KeyAction::Transparent => DEFAULT_TRANSPARENT.to_string(),
        KeyAction::NoOp => DEFAULT_NO_OP.to_string(),
        KeyAction::LayerMomentary(l) => format!("MO({l})"),
        KeyAction::LayerToggle(l) => format!("TG({l})"),
        KeyAction::LayerOn(l) => format!("TO({l})"),
        KeyAction::ModTap { mod_name, key } => {
            let key_str = action_to_qmk(key, registry);
            format!("{}_T({})", util::sanitize_c(mod_name), key_str)
        }
        KeyAction::LayerTap { layer, key } => {
            let key_str = action_to_qmk(key, registry);
            format!("LT({layer}, {key_str})")
        }
        KeyAction::StickyMod(m) => {
            let qmk_mod = util::map_modifier(m, ModFormat::Qmk);
            format!("OSM({qmk_mod})")
        }
        KeyAction::CapsWord => "CAPS_WORD".to_string(),
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
        let result = exporter.generate("Test Layout", &layers, None).unwrap();

        assert!(result.contains("keymaps[][MATRIX_ROWS][MATRIX_COLS]"));
        assert!(result.contains("[0] = LAYOUT("));
        assert!(result.contains("[1] = LAYOUT("));
        assert!(result.contains("_______"));
        assert!(result.contains("XXXXXXX"));
    }

    #[test]
    fn test_action_to_qmk_all() {
        assert_eq!(
            action_to_qmk(&KeyAction::LayerMomentary(1), None),
            "MO(1)"
        );
        assert_eq!(action_to_qmk(&KeyAction::LayerToggle(2), None), "TG(2)");
        assert_eq!(action_to_qmk(&KeyAction::LayerOn(3), None), "TO(3)");
        assert_eq!(action_to_qmk(&KeyAction::CapsWord, None), "CAPS_WORD");

        let mt = KeyAction::ModTap {
            mod_name: "LSFT".into(),
            key: Box::new(KeyAction::Simple("Z".into())),
        };
        assert_eq!(action_to_qmk(&mt, None), "LSFT_T(Z)");

        let lt = KeyAction::LayerTap {
            layer: 1,
            key: Box::new(KeyAction::Simple("SPC".into())),
        };
        assert_eq!(action_to_qmk(&lt, None), "LT(1, SPC)");

        let sk = KeyAction::StickyMod("LSHIFT".into());
        assert_eq!(action_to_qmk(&sk, None), "OSM(MOD_LSFT)");
    }

    #[test]
    fn test_qmk_generate_long_lines() {
        let exporter = QmkExporter;
        let layers = vec![vec!["A".to_string(); 20]];
        let result = exporter.generate("Long", &layers, None).unwrap();
        assert!(result.contains("\n    ")); // Newline inserted after 12 keys
    }

    #[test]
    fn test_qmk_generate_errors() {
        let exporter = QmkExporter;

        // 1. Too many keys
        let layers = vec![vec!["A".into(); MAX_KEYS + 1]];
        assert!(exporter.generate("fail", &layers, None).is_err());

        // 2. Output size limit
        // We use exactly MAX_KEYS but with very long labels to exceed 64KB
        let layers = vec![vec!["A".repeat(300); MAX_KEYS]];
        let res = exporter.generate("big", &layers, None);
        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .to_string()
            .contains("Output size limit exceeded"));
    }
}
