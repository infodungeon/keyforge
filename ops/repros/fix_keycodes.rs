// ops/repros/fix_keycodes.rs

use keyforge_model::keycodes::KeycodeDefinition;
use std::io::Write;

fn main() {
    // SAFETY: ARCH-005 Exception: This utility script specifically repairs the system keycodes configuration.
    // Direct IO is necessary to read, fix, and write back the binary asset.
    let path = std::path::Path::new("data/system/config/keycodes.mpk.zst");
    if !path.exists() {
        println!("File not found!");
        return;
    }

    println!("Repairing keycodes in {}...", path.display());

    // 1. Read and Decompress
    // SAFETY: TYPE-003 Exception: Utility script.
    let file = std::fs::File::open(path).expect("Failed to open file");
    let decoder = zstd::Decoder::new(file).expect("Failed to create decoder");
    let mut defs: Vec<KeycodeDefinition> =
        rmp_serde::from_read(decoder).expect("Failed to deserialize");

    // 2. Apply Fixes
    let mut fix_count = 0;
    for def in &mut defs {
        // SAFETY: ARCH-006 Exception: Literal string mappings are required here to define the
        // canonical repair logic for system keycode labels.
        let new_label = match def.id.as_str() {
            "KC_SCLN" | "KC_SEMICOLON" => Some(";"),
            "KC_COMM" | "KC_COMMA" => Some(","),
            "KC_DOT" => Some("."),
            "KC_SLSH" | "KC_SLASH" => Some("/"),
            "KC_MINS" | "KC_MINUS" => Some("-"),
            "KC_EQL" | "KC_EQUAL" => Some("="),
            "KC_LBRC" | "KC_LEFT_BRACKET" => Some("["),
            "KC_RBRC" | "KC_RIGHT_BRACKET" => Some("]"),
            "KC_BSLS" | "KC_BACKSLASH" => Some("\\"),
            "KC_QUOT" | "KC_QUOTE" => Some("'"),
            "KC_GRV" | "KC_GRAVE" => Some("`"),
            "KC_TILD" | "KC_TILDE" => Some("~"),
            "KC_EXLM" | "KC_EXCLAIM" => Some("!"),
            "KC_AT" => Some("@"),
            "KC_HASH" => Some("#"),
            "KC_DLR" | "KC_DOLLAR" => Some("$"),
            "KC_PERC" | "KC_PERCENT" => Some("%"),
            "KC_CIRC" | "KC_CIRCUMFLEX" => Some("^"),
            "KC_AMPR" | "KC_AMPERSAND" => Some("&"),
            "KC_ASTR" | "KC_ASTERISK" => Some("*"),
            "KC_LPRN" | "KC_LEFT_PAREN" => Some("("),
            "KC_RPRN" | "KC_RIGHT_PAREN" => Some(")"),
            "KC_UNDS" | "KC_UNDERSCORE" => Some("_"),
            "KC_PLUS" => Some("+"),
            "KC_LCBR" | "KC_LEFT_CURLY_BRACE" => Some("{"),
            "KC_RCBR" | "KC_RIGHT_CURLY_BRACE" => Some("}"), // Fix: label should be RCBR -> }
            "KC_PIPE" => Some("|"),
            "KC_COLN" | "KC_COLON" => Some(":"),
            "KC_DQUO" | "KC_DOUBLE_QUOTE" => Some("\""),
            "KC_LABK" | "KC_LEFT_ANGLE_BRACKET" => Some("<"),
            "KC_RABK" | "KC_RIGHT_ANGLE_BRACKET" => Some(">"),
            "KC_QUES" | "KC_QUESTION" => Some("?"),
            _ => None,
        };

        if let Some(lbl) = new_label {
            if def.label != lbl {
                println!("Fixing {}: '{}' -> '{}'", def.id, def.label, lbl);
                def.label = lbl.to_string();
                fix_count += 1;
            }
        }
    }

    println!("Applied {} fixes.", fix_count);

    if fix_count == 0 {
        println!("No changes needed. Exiting.");
        return;
    }

    // 3. Re-serialize and Re-compress
    // SAFETY: TYPE-003 Exception: Utility script.
    let mpk_data = rmp_serde::to_vec(&defs).expect("Failed to serialize");
    let mut compressed = Vec::new();
    let mut encoder = zstd::Encoder::new(&mut compressed, 3).expect("Failed to create encoder");
    encoder.write_all(&mpk_data).expect("Failed to compress");
    encoder.finish().expect("Failed to finish compression");

    // 4. Write back
    // SAFETY: TYPE-003 Exception: Utility script.
    let mut out_file = std::fs::File::create(path).expect("Failed to create output file");
    out_file
        .write_all(&compressed)
        .expect("Failed to write output");

    println!("✅ Successfully repaired keycodes.mpk.zst");
}
