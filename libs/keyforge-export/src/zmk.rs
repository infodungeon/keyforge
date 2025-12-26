use crate::Exporter;
use anyhow::Result;
use keyforge_adapter::parsing::{parse_key, KeyAction};

pub struct ZmkExporter;

fn sanitize(s: &str) -> String {
    s.replace(|c: char| !c.is_alphanumeric() && c != '_', "")
}

impl Exporter for ZmkExporter {
    fn generate(&self, layout_name: &str, keys: &[String]) -> Result<String> {
        let mut out = String::new();

        out.push_str(&format!("// KeyForge ZMK Export: {}\n", layout_name));
        out.push_str(&format!(
            "// Generated at: {}\n\n",
            chrono::Local::now().to_rfc3339()
        ));

        out.push_str("/ {\n");
        out.push_str("    keymap {\n");
        out.push_str("        compatible = \"zmk,keymap\";\n\n");
        out.push_str("        default_layer {\n");
        out.push_str("            bindings = <\n");
        out.push_str("                ");

        let mut line_len = 0;
        for (i, key_str) in keys.iter().enumerate() {
            let action = parse_key(key_str);
            let code = match action {
                KeyAction::Simple(s) => {
                    let clean = s.strip_prefix("KC_").unwrap_or(&s);
                    format!("&kp {}", sanitize(clean))
                }
                KeyAction::Transparent => "&trans".to_string(),
                KeyAction::NoOp => "&none".to_string(),
                KeyAction::LayerMomentary(l) => format!("&mo {}", l),
                KeyAction::LayerToggle(l) => format!("&tog {}", l),
                KeyAction::LayerOn(l) => format!("&to {}", l),
                KeyAction::ModTap { mod_name, key } => {
                    let zmk_mod = match mod_name.as_str() {
                        "LSFT" | "SFT" => "LSHIFT",
                        "RSFT" => "RSHIFT",
                        "LCTL" | "CTL" => "LCTRL",
                        "RCTL" => "RCTRL",
                        "LALT" | "ALT" => "LALT",
                        "RALT" | "ALGR" => "RALT",
                        "LGUI" | "GUI" | "WIN" | "CMD" => "LGUI",
                        "RGUI" => "RGUI",
                        _ => &mod_name,
                    };
                    let clean_key = key.strip_prefix("KC_").unwrap_or(&key);
                    format!("&mt {} {}", sanitize(zmk_mod), sanitize(clean_key))
                }
                KeyAction::LayerTap { layer, key } => {
                    let clean_key = key.strip_prefix("KC_").unwrap_or(&key);
                    format!("&lt {} {}", layer, sanitize(clean_key))
                }
                KeyAction::StickyMod(m) => {
                    let zmk_mod = match m.as_str() {
                        "LSFT" | "SFT" => "LSHIFT",
                        "RSFT" => "RSHIFT",
                        "LCTL" | "CTL" => "LCTRL",
                        "RCTL" => "RCTRL",
                        "LALT" | "ALT" => "LALT",
                        "RALT" | "ALGR" => "RALT",
                        "LGUI" | "GUI" | "WIN" | "CMD" => "LGUI",
                        "RGUI" => "RGUI",
                        _ => &m,
                    };
                    format!("&sk {}", sanitize(zmk_mod))
                }
                KeyAction::CapsWord => "&caps_word".to_string(),
                KeyAction::Raw(s) => sanitize(&s),
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
        out.push_str("    };\n");
        out.push_str("};\n");

        Ok(out)
    }
}
