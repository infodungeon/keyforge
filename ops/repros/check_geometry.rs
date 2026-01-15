use std::fs::File;
use keyforge_model::geometry::KeyboardDefinition;
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

    for (i, key) in kb.geometry.keys.iter().enumerate() {
        println!("Key {:<2}: Label='{:<8}' Finger={:?} Hand={:?}", i, key.label, key.finger, key.hand);
    }
}
