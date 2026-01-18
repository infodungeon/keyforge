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

/// An exporter for the ZMK (Zephyr Mechanical Keyboard) firmware.
///
/// This generates a Devicetree (.keymap) file compatible with ZMK's build system.
#[derive(Debug)]
pub struct ZmkExporter;

impl Exporter for ZmkExporter {
    fn generate(&self, layout_name: &str, layers: &[Vec<String>]) -> Result<String> {
        let mut out = String::new();

        use std::fmt::Write;
        let _ = write!(out, "// KeyForge ZMK Export: {layout_name}\n");
        let _ = write!(
            out,
            "// Generated at: {}\n\n",
            chrono::Local::now().to_rfc3339()
        );

        out.push_str("/ {\n");
        out.push_str("    keymap {\n");
        out.push_str("        compatible = \"zmk,keymap\";\n\n");

        for (l_idx, keys) in layers.iter().enumerate() {
            let layer_name = if l_idx == 0 {
                "default_layer".to_string()
            } else {
                format!("layer_{l_idx}")
            };

            let _ = write!(out, "        {layer_name} {{\n");
            out.push_str("            bindings = <\n");
            out.push_str("                ");

            let mut line_len = 0;
            for (i, key_str) in keys.iter().enumerate() {
                let action = parse_key(key_str);
                let code = match action {
                    KeyAction::Simple(s) => {
                        let clean = s.strip_prefix("KC_").unwrap_or(&s);
                        format!("&kp {}", util::sanitize_zmk(clean))
                    }
                    KeyAction::Transparent => "&trans".to_string(),
                    KeyAction::NoOp => "&none".to_string(),
                    KeyAction::LayerMomentary(l) => format!("&mo {l}"),
                    KeyAction::LayerToggle(l) => format!("&tog {l}"),
                    KeyAction::LayerOn(l) => format!("&to {l}"),
                    KeyAction::ModTap { mod_name, key } => {
                        let zmk_mod = util::map_modifier(&mod_name, ModFormat::Zmk);
                        let key_str = match key.as_ref() {
                            KeyAction::Simple(s) | KeyAction::Raw(s) => s.clone(),
                            _ => "failed_recursion".to_string(),
                        };
                        let clean_key = key_str.strip_prefix("KC_").unwrap_or(&key_str);
                        format!(
                            "&mt {} {}",
                            util::sanitize_zmk(&zmk_mod),
                            util::sanitize_zmk(clean_key)
                        )
                    }
                    KeyAction::LayerTap { layer, key } => {
                        let key_str = match key.as_ref() {
                            KeyAction::Simple(s) | KeyAction::Raw(s) => s.clone(),
                            _ => "failed_recursion".to_string(),
                        };
                        let clean_key = key_str.strip_prefix("KC_").unwrap_or(&key_str);
                        format!("&lt {} {}", layer, util::sanitize_zmk(clean_key))
                    }
                    KeyAction::StickyMod(m) => {
                        let zmk_mod = util::map_modifier(&m, ModFormat::Zmk);
                        format!("&sk {}", util::sanitize_zmk(&zmk_mod))
                    }
                    KeyAction::CapsWord => "&caps_word".to_string(),
                    KeyAction::Raw(s) => util::sanitize_zmk(&s),
                };

                out.push_str(&code);

                if i < keys.len() - 1 {
                    out.push(' ');
                }

                line_len += 1;
                if line_len >= 12 {
                    out.push_str("\n                ");
                    line_len = 0;
                }
            }

            out.push_str("\n            >;\n");
            out.push_str("        };\n");
        }

        out.push_str("    };\n");
        out.push_str("};\n");

        Ok(out)
    }
}
