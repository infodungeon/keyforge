#![allow(clippy::unwrap_used, clippy::expect_used)]
use std::fs::File;
use std::io::{Read, Write};

fn main() {
    println!("CWD: {:?}", std::env::current_dir().unwrap());
    let json_path = "data/system/config/keycodes.json";
    let mpk_path = "data/system/config/keycodes.mpk.zst";

    println!("Compiling {} to {}...", json_path, mpk_path);

    // 1. Read JSON
    let mut file = File::open(json_path).expect("Failed to open JSON file");
    let mut json_data = String::new();
    file.read_to_string(&mut json_data)
        .expect("Failed to read JSON file");

    let registry_dto: keyforge_protocol::KeycodeRegistryDto =
        serde_json::from_str(&json_data).expect("Failed to parse JSON");

    // 2. Encode to MessagePack
    let mpk_data = rmp_serde::to_vec(&registry_dto).expect("Failed to encode to MessagePack");

    // 3. Compress with Zstd
    let mut compressed = Vec::new();
    let mut encoder = zstd::Encoder::new(&mut compressed, 3).expect("Failed to create encoder");
    encoder.write_all(&mpk_data).expect("Failed to compress");
    encoder.finish().expect("Failed to finish compression");

    // 4. Write to file
    let mut out_file = File::create(mpk_path).expect("Failed to create output file");
    out_file
        .write_all(&compressed)
        .expect("Failed to write output file");

    println!("✅ Successfully compiled and compressed keycodes.");
}
