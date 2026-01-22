// ops/repros/generate_keycodes.rs
//
// This script parses the `docs/data/QMK_Keycodes.md` file and generates a
// JSON-compatible `KeycodeRegistry`. This automates the maintenance of
// the keycode database and ensures alignment with QMK documentation.

use keyforge_model::keycodes::{KeycodeDefinition, KeycodeRegistry};
use keyforge_model::types::KeyCode;
use regex::Regex;
use std::fs;
use std::path::Path;

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let docs_path = Path::new(manifest_dir).join("../../docs/data/QMK_Keycodes.md");

    if !docs_path.exists() {
        eprintln!("Error: Docs not found at {}", docs_path.display());
        std::process::exit(1);
    }

    let content = fs::read_to_string(&docs_path).expect("Failed to read docs");
    let definitions = parse_markdown(&content);

    let registry = KeycodeRegistry::new(definitions);
    
    // Validate
    if let Err(e) = keyforge_model::validator::Validator::validate(&registry) {
        eprintln!("Validation Error: {}", e);
        std::process::exit(1);
    }

    let json = serde_json::to_string_pretty(&registry).expect("Failed to serialize");
    println!("{}", json);
}

fn parse_markdown(content: &str) -> Vec<KeycodeDefinition> {
    let mut defs = Vec::new();
    let row_regex = Regex::new(r"^\|\s*`([^`]+)`\s*\|\s*([^|]*)\|\s*([^|]+)\|").unwrap();
    let code_regex = Regex::new(r"`([^`]+)`").unwrap();

    // Start with special internal keys
    defs.push(KeycodeDefinition {
        code: KeyCode(0),
        id: "KC_NO".into(),
        label: " ".into(),
        aliases: vec!["XXXXXXX".into()],
    });
    defs.push(KeycodeDefinition {
        code: KeyCode(1),
        id: "KC_TRANSPARENT".into(),
        label: "▽".into(),
        aliases: vec!["KC_TRNS".into(), "_______".into()],
    });

    let mut next_code = 4; // QMK starts A at 4

    for line in content.lines() {
        if let Some(caps) = row_regex.captures(line) {
            let id = caps[1].trim().to_string();
            
            // Skip already added
            if id == "KC_NO" || id == "KC_TRANSPARENT" {
                continue;
            }

            let aliases_str = caps[2].trim();
            let desc = caps[3].trim().to_string();

            let mut aliases = Vec::new();
            for m in code_regex.find_iter(aliases_str) {
                let alias = m.as_str().trim_matches('`').to_string();
                if alias != id {
                    aliases.push(alias);
                }
            }

            // Heuristic label generation
            let label = if id.starts_with("KC_") && id.len() == 4 {
                // KC_A -> A
                id.chars().last().unwrap().to_string()
            } else if let Some(alias) = aliases.first() {
                // Use first alias if short
                if alias.len() < 5 {
                    alias.clone()
                } else {
                    id.clone() // Fallback
                }
            } else {
                // Use ID suffix
                id.strip_prefix("KC_").unwrap_or(&id).to_string()
            };

            defs.push(KeycodeDefinition {
                code: KeyCode(next_code),
                id,
                label,
                aliases,
            });

            next_code += 1;
        }
    }
    defs
}
