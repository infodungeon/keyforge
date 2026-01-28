#![allow(clippy::unwrap_used, clippy::expect_used)]
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::layout::LayoutCatalog;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("data/system/keyboards/models/szr35.mpk.zst")?;
    let decoder = zstd::Decoder::new(file)?;
    let _kb: KeyboardDefinition = rmp_serde::from_read(decoder)?;

    let cat_file = File::open("data/system/layouts/szr35.mpk.zst")?;
    let cat_decoder = zstd::Decoder::new(cat_file)?;
    let catalog: LayoutCatalog = rmp_serde::from_read(cat_decoder)?;

    if let Some(layout) = catalog.layouts.get("Colemak-DH") {
        println!("Colemak-DH Layout for SZR35:");
        let keys: Vec<&str> = layout.split_whitespace().collect();
        for (i, k) in keys.iter().enumerate() {
            println!("  Idx {:2}: {}", i, k);
        }
    } else {
        println!(
            "colemak-dh not found in szr35. Available: {:?}",
            catalog.layouts.keys()
        );
    }

    Ok(())
}
