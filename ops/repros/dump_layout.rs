use keyforge_model::geometry::KeyboardDefinition;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("data/system/keyboards/models/szr35.mpk.zst")?;
    let decoder = zstd::Decoder::new(file)?;
    let kb: KeyboardDefinition = rmp_serde::from_read(decoder)?;

    if let Some(layout) = kb.layouts.get("Colemak-DH") {
        println!("Colemak-DH Layout for SZR35:");
        let keys: Vec<&str> = layout.split_whitespace().collect();
        for (i, k) in keys.iter().enumerate() {
            println!("  Idx {:2}: {}", i, k);
        }
    } else {
        println!("colemak-dh not found in szr35. Available: {:?}", kb.layouts.keys());
    }

    Ok(())
}
