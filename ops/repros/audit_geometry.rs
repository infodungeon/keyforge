use keyforge_model::geometry::KeyboardDefinition;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("data/system/keyboards/models/szr35.mpk.zst")?;
    let decoder = zstd::Decoder::new(file)?;
    let kb: KeyboardDefinition = rmp_serde::from_read(decoder)?;

    println!("Keyboard: {}", kb.meta.name);
    println!("Home Row: {}", kb.geometry.home_row);
    println!("Keys:");
    for (i, k) in kb.geometry.keys.iter().enumerate() {
        println!(
            "  Idx {:2}: x={:5.2}, y={:5.2}, h={:1}, f={:1}, r={:1}, c={:2}, home={:5}, label={}",
            i,
            k.x,
            k.y,
            k.hand.as_u8(),
            k.finger.as_u8(),
            k.row.0,
            k.col.0,
            k.is_home,
            k.label
        );
    }

    Ok(())
}
