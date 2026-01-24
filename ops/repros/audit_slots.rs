#![allow(clippy::unwrap_used, clippy::expect_used)]
use keyforge_model::geometry::KeyboardDefinition;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("data/system/keyboards/models/szr35.mpk.zst")?;
    let decoder = zstd::Decoder::new(file)?;
    let kb: KeyboardDefinition = rmp_serde::from_read(decoder)?;

    println!("SZR35 Slots:");
    print!("Prime: ");
    for s in kb.geometry.prime_slots { print!("{} ", s); }
    println!("\nMed:   ");
    for s in kb.geometry.med_slots { print!("{} ", s); }
    println!("\nLow:   ");
    for s in kb.geometry.low_slots { print!("{} ", s); }
    println!();

    Ok(())
}
