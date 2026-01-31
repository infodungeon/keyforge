#![allow(clippy::unwrap_used, clippy::expect_used)]
use keyforge_model::geometry::KeyboardDefinition;
use std::fs::File;
use std::path::Path;

fn main() {
    let path = Path::new("data/system/keyboards/models/szr35.mpk.zst");
    if !path.exists() {
        println!("File not found!");
        return;
    }

    let file = File::open(path).expect("Failed to open file");
    let decoder = zstd::Decoder::new(file).expect("Failed to create decoder");
    let kb: KeyboardDefinition = rmp_serde::from_read(decoder).expect("Failed to deserialize");

    println!("Keyboard: {}", kb.meta.name);
    println!("Keys: {}", kb.geometry.keys.len());
    println!("Home Row: {}", kb.geometry.home_row);
    println!(
        "{:<3} {:<8} {:<8} {:<8} {:<8} {:<8} {:<8} {:<8}",
        "Idx", "Label", "Finger", "Hand", "Row", "Col", "Home", "Pos"
    );

    for (i, key) in kb.geometry.keys.iter().enumerate() {
        println!(
            "{:<3} {:<8} {:<8?} {:<8?} {:<8} {:<8} {:<8} ({:.1}, {:.1})",
            i,
            key.label,
            key.finger,
            key.hand,
            key.row.raw(),
            key.col.raw(),
            key.is_home,
            key.x.to_f32(),
            key.y.to_f32()
        );
    }
}
