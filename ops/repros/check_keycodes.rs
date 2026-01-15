use keyforge_model::keycodes::KeycodeRegistry;
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
    let reg: KeycodeRegistry = rmp_serde::from_read(decoder).unwrap();

    println!("Definitions: {}", reg.definitions.len());
    
    if let Some(code) = reg.get_code("KC_ESC") {
        println!("KC_ESC found: {}", code);
    } else {
        println!("KC_ESC NOT found!");
        for def in reg.definitions.iter().take(10) {
            println!("  ID: {}, Code: {}", def.id, def.code);
        }
    }
}
