#![allow(clippy::unwrap_used, clippy::expect_used)]
use keyforge_model::geometry::KeyboardDefinition;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    let path = Path::new("data/system/keyboards/models/szr35.mpk.zst");
    if !path.exists() {
        println!("File not found!");
        return;
    }

    println!("Repairing SZR35 geometry in {}...", path.display());

    // 1. Read and Decompress
    let file = File::open(path).expect("Failed to open file");
    let decoder = zstd::Decoder::new(file).expect("Failed to create decoder");
    let mut kb_def: KeyboardDefinition =
        rmp_serde::from_read(decoder).expect("Failed to deserialize");

    // 2. Identify and Set Home Keys
    // Based on user request and standard Colemak-DH home row (Row 1)
    // and thumb positions (Row 3, Space keys)

    let mut fix_count = 0;
    for (i, key) in kb_def.geometry.keys.iter_mut().enumerate() {
        let should_be_home = match i {
            // Left Hand Home Row (A S D F)
            5..=8 => true,
            // Left Thumb (SpaceL)
            16 => true,
            // Right Hand Home Row (J K L ;)
            24..=27 => true,
            // Right Thumb (SpaceR)
            34 => true,
            _ => false,
        };

        if key.is_home != should_be_home {
            println!(
                "Setting Key {} ({}): is_home = {}",
                i, key.label, should_be_home
            );
            key.is_home = should_be_home;
            fix_count += 1;
        }
    }

    println!("Applied {} changes.", fix_count);

    if fix_count == 0 {
        println!("No changes needed. Exiting.");
        return;
    }

    // 3. Re-serialize and Re-compress
    let mpk_data = rmp_serde::to_vec(&kb_def).expect("Failed to serialize");
    let mut compressed = Vec::new();
    let mut encoder = zstd::Encoder::new(&mut compressed, 3).expect("Failed to create encoder");
    encoder.write_all(&mpk_data).expect("Failed to compress");
    encoder.finish().expect("Failed to finish compression");

    // 4. Write back
    let mut out_file = File::create(path).expect("Failed to create output file");
    out_file
        .write_all(&compressed)
        .expect("Failed to write output");

    println!("✅ Successfully repaired szr35.mpk.zst");
}
