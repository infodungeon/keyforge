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
use keyforge_adapter::parsing::{parse_key, KeyAction};

pub struct QmkExporter;

fn sanitize(s: &str) -> String {
    // Strict allowlist for C identifiers and macros.
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '(' || *c == ')' || *c == ',')
        .collect()
}

const MAX_OUTPUT_SIZE: usize = 1024 * 1024; // 1MB Limit
const MAX_KEYS: usize = 200; // Physical limit for standard boards

impl Exporter for QmkExporter {
    fn generate(&self, layout_name: &str, keys: &[String]) -> Result<String> {
        if keys.len() > MAX_KEYS {
            return Err(anyhow::anyhow!(
                "Too many keys for QMK export (Limit: {})",
                MAX_KEYS
            ));
        }

        let mut out = String::with_capacity(4096);
        // Sanitize layout name for C identifier
        let safe_name = layout_name.replace(" ", "_").to_uppercase();
        let safe_name = sanitize(&safe_name);

        out.push_str(&format!("// KeyForge QMK Export: {}\n", layout_name));
        out.push_str(&format!(
            "// Generated at: {}\n\n",
            chrono::Local::now().to_rfc3339()
        ));

        out.push_str("#include QMK_KEYBOARD_H\n\n");
        out.push_str("const uint16_t PROGMEM keymaps[][MATRIX_ROWS][MATRIX_COLS] = {\n");
        out.push_str(&format!("  [{}] = LAYOUT(\n", safe_name));
        out.push_str("    ");

        let mut line_len = 0;
        for (i, key_str) in keys.iter().enumerate() {
            // Safety Check
            if out.len() > MAX_OUTPUT_SIZE {
                return Err(anyhow::anyhow!("Output size limit exceeded"));
            }

            let action = parse_key(key_str);
            let code = match action {
                KeyAction::Simple(s) => sanitize(&s),
                KeyAction::Transparent => "_______".to_string(),
                KeyAction::NoOp => "XXXXXXX".to_string(),
                KeyAction::LayerMomentary(l) => format!("MO({})", l),
                KeyAction::LayerToggle(l) => format!("TG({})", l),
                KeyAction::LayerOn(l) => format!("TO({})", l),
                KeyAction::ModTap { mod_name, key } => {
                    format!("{}_T({})", sanitize(&mod_name), sanitize(&key))
                }
                KeyAction::LayerTap { layer, key } => format!("LT({}, {})", layer, sanitize(&key)),
                KeyAction::StickyMod(m) => {
                    let mod_str = sanitize(&m);
                    let qmk_mod = match mod_str.as_str() {
                        "LSFT" | "SFT" | "LSHIFT" => "MOD_LSFT",
                        "RSFT" | "RSHIFT" => "MOD_RSFT",
                        "LCTL" | "CTL" | "LCTRL" => "MOD_LCTL",
                        "RCTL" | "RCTRL" => "MOD_RCTL",
                        "LALT" | "ALT" => "MOD_LALT",
                        "RALT" | "ALGR" => "MOD_RALT",
                        "LGUI" | "GUI" | "WIN" | "CMD" => "MOD_LGUI",
                        "RGUI" => "MOD_RGUI",
                        _ => &mod_str,
                    };
                    format!("OSM({})", qmk_mod)
                }
                KeyAction::CapsWord => "CAPS_WORD".to_string(),
                KeyAction::Raw(s) => sanitize(&s),
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
        out.push_str("};\n");
        Ok(out)
    }
}
