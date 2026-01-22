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

use crate::util::{self, ModFormat};
use crate::Exporter;
use anyhow::Result;
use keyforge_adapter::parsing::{parse_key, KeyAction};
use std::fmt::Write;

/// An exporter for the ZMK (Zephyr Mechanical Keyboard) firmware.
///
/// This generates a Devicetree (.keymap) file compatible with ZMK's build system.
#[derive(Debug)]
pub struct ZmkExporter;

impl Exporter for ZmkExporter {
    fn generate(&self, layout_name: &str, layers: &[Vec<String>]) -> Result<String> {
        let mut out = String::new();

        let _ = writeln!(out, "// KeyForge ZMK Export: {layout_name}");
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

            let _ = writeln!(out, "        {layer_name} {{");
            out.push_str("            bindings = <\n");
            out.push_str("                ");

            let mut line_len = 0;
            for (i, key_str) in keys.iter().enumerate() {
                let action = parse_key(key_str);
                let code = match action {
                    Ok(KeyAction::Simple(s)) => {
                        let clean = s.strip_prefix("KC_").unwrap_or(&s);
                        format!("&kp {}", util::sanitize_zmk(clean))
                    }
                    Ok(KeyAction::Transparent) => "&trans".to_string(),
                    Ok(KeyAction::NoOp) => "&none".to_string(),
                    Ok(KeyAction::LayerMomentary(l)) => format!("&mo {l}"),
                    Ok(KeyAction::LayerToggle(l)) => format!("&tog {l}"),
                    Ok(KeyAction::LayerOn(l)) => format!("&to {l}"),
                    Ok(KeyAction::ModTap { mod_name, key }) => {
                        let zmk_mod = util::map_modifier(&mod_name, ModFormat::Zmk);
                        let key_str = match key.as_ref() {
                            KeyAction::Simple(s) => s.clone(),
                            _ => "failed_recursion".to_string(),
                        };
                        let clean_key = key_str.strip_prefix("KC_").unwrap_or(&key_str);
                        format!(
                            "&mt {} {}",
                            util::sanitize_zmk(&zmk_mod),
                            util::sanitize_zmk(clean_key)
                        )
                    }
                    Ok(KeyAction::LayerTap { layer, key }) => {
                        let key_str = match key.as_ref() {
                            KeyAction::Simple(s) => s.clone(),
                            _ => "failed_recursion".to_string(),
                        };
                        let clean_key = key_str.strip_prefix("KC_").unwrap_or(&key_str);
                        format!("&lt {} {}", layer, util::sanitize_zmk(clean_key))
                    }
                    Ok(KeyAction::StickyMod(m)) => {
                        let zmk_mod = util::map_modifier(&m, ModFormat::Zmk);
                        format!("&sk {}", util::sanitize_zmk(&zmk_mod))
                    }
                    Ok(KeyAction::CapsWord) => "&caps_word".to_string(),
                    Err(_) => {
                        // Fallback to simple if parsing failed
                        format!("&kp {}", util::sanitize_zmk(key_str))
                    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zmk_generate_all_actions() {
        let exporter = ZmkExporter;
        let layers = vec![vec![
            "A".to_string(),
            "TRNS".to_string(),
            "NO".to_string(),
            "MO(1)".to_string(),
            "TG(2)".to_string(),
            "TO(3)".to_string(),
            "MT(LSFT, Z)".to_string(),
            "LT(1, SPC)".to_string(),
            "SK(LCTL)".to_string(),
            "CAPS_WORD".to_string(),
            "UNKNOWN(TOKEN)".to_string(),
        ]];
        let result = exporter.generate("Test", &layers).unwrap();

        assert!(result.contains("&kp A"));
        assert!(result.contains("&trans"));
        assert!(result.contains("&none"));
        assert!(result.contains("&mo 1"));
        assert!(result.contains("&tog 2"));
        assert!(result.contains("&to 3"));
        assert!(result.contains("&mt LSHIFT Z"));
        assert!(result.contains("&lt 1 SPC"));
        assert!(result.contains("&sk LCTRL"));
        assert!(result.contains("&caps_word"));
        assert!(result.contains("UNKNOWNTOKEN")); // Sanitize removes brackets
    }

    #[test]
    fn test_zmk_recursion_failure() {
        let exporter = ZmkExporter;
        // MT with a nested MT
        let layers = vec![vec!["MT(MOD_LSFT, MT(MOD_LCTL, Z))".into()]];
        let result = exporter.generate("RecursionFail", &layers).unwrap();
        assert!(result.contains("failed_recursion"));

        // LT with a nested LT
        let layers = vec![vec!["LT(1, LT(2, Z))".into()]];
        let result = exporter.generate("RecursionFail", &layers).unwrap();
        assert!(result.contains("failed_recursion"));
    }

    #[test]
    fn test_zmk_multi_layer() {
        let exporter = ZmkExporter;
        let layers = vec![vec!["A".into()], vec!["B".into()]];
        let result = exporter.generate("Multi", &layers).unwrap();
        assert!(result.contains("default_layer"));
        assert!(result.contains("layer_1"));
    }

    #[test]
    fn test_zmk_generate_long_lines() {
        let exporter = ZmkExporter;
        let layers = vec![vec!["A".to_string(); 20]];
        let result = exporter.generate("Long", &layers).unwrap();
        assert!(result.contains("\n                ")); // Newline after 12 keys
    }
}
