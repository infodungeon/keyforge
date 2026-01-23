use std::fs::File;

fn main() {
    let path = "data/system/config/ui_categories.mpk.zst";
    let file = File::open(path).expect("Failed to open file");
    let decoder = zstd::Decoder::new(file).expect("Failed to create decoder");
    let json: serde_json::Value = rmp_serde::from_read(decoder).expect("Failed to deserialize");
    println!("{}", serde_json::to_string_pretty(&json).unwrap());
}
