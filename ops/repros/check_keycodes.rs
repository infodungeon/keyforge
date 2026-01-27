#![allow(clippy::unwrap_used, clippy::expect_used)]
use keyforge_model::keycodes::{KeycodeDefinition, KeycodeRegistry};
use std::fs::File;
use std::path::Path;

fn main() {
    let path = Path::new("data/system/config/keycodes.mpk.zst");
    if !path.exists() {
        println!("File not found!");
        return;
    }

    let file = File::open(path).unwrap();
    let decoder = zstd::Decoder::new(file).unwrap();
    // Fix: The file contains Vec<KeycodeDefinition>, not KeycodeRegistry struct
    let defs: Vec<KeycodeDefinition> = rmp_serde::from_read(decoder).unwrap();

    // KeycodeRegistry::new() performs the QMK -> ASCII remapping
    let reg = KeycodeRegistry::new(defs);

    println!("--- Verifying Fixed Keys ---");
    let targets = [
        "KC_ESCAPE",
        "KC_ESC",
        "KC_BACKSPACE",
        "KC_BSPC",
        "KC_SPACE",
        "KC_SPC",
        "KC_ENTER",
        "KC_ENT",
        "KC_NO",
        "KC_SCLN",
        "KC_SEMICOLON",
        "KC_COMM",
        "KC_COMMA",
        "KC_DOT",
        "KC_SLSH",
        "KC_SLASH",
        "KC_MINUS",
        "KC_MINS",
        "KC_EQUAL",
        "KC_EQL",
        "KC_LBRC",
        "KC_LEFT_BRACKET",
        "KC_RBRC",
        "KC_RIGHT_BRACKET",
        "KC_BSLS",
        "KC_BACKSLASH",
        "KC_QUOTE",
        "KC_QUOT",
        "KC_GRAVE",
        "KC_GRV",
        "KC_1",
        "KC_2",
        "KC_3",
        "KC_4",
        "KC_5",
        "KC_6",
        "KC_7",
        "KC_8",
        "KC_9",
        "KC_0",
    ];
    for t in targets {
        if let Some(code) = reg.get_code(t) {
            let label = reg.get_label(code);
            println!("Target: {:<15} Code: {:<5} Label: '{}'", t, code, label);
        } else {
            println!("Target: {:<15} NOT FOUND", t);
        }
    }

    println!("\n--- Scanning for Suspicious Labels ---");
    let mut suspicious_count = 0;
    for def in &reg.definitions {
        // ID is long (e.g. KC_SOMETHING) but label is short (e.g. 'a')
        if def.id.len() > 3 && def.label.len() == 1 {
            // Filter out expected ones like KC_1 -> 1, KC_A -> a
            let char_code = def.label.chars().next().unwrap();
            if char_code.is_alphanumeric() {
                // Check if ID ends with the char (KC_A ends with A)
                if !def.id.ends_with(&def.label.to_uppercase()) {
                    println!(
                        "Suspicious: ID={:<15} Label='{}' Code={}",
                        def.id, def.label, def.code
                    );
                    suspicious_count += 1;
                }
            }
        }
    }

    if suspicious_count == 0 {
        println!("No other suspicious labels found.");
    } else {
        println!("Found {} suspicious labels.", suspicious_count);
    }
}
