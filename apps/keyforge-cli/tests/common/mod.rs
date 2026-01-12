use std::path::PathBuf;

/// Locates the compiled binary for integration testing.
pub fn get_binary_path() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let mut path = PathBuf::from(manifest_dir);
    
    // Handle running from workspace root vs crate root
    if path.ends_with("keyforge-cli") {
        path.pop();
        path.pop();
    }
    
    path.push("target");

    // Check debug first (faster for tests), then release
    let debug_path = path.join("debug").join("keyforge-cli");
    if debug_path.exists() {
        return debug_path;
    }
    
    let release_path = path.join("release").join("keyforge-cli");
    if release_path.exists() {
        return release_path;
    }
    
    // Windows fallback
    path.join("debug").join("keyforge-cli.exe")
}

/// Helper to extract the score from CLI stderr output.
#[allow(dead_code)]
pub fn extract_score(output: &str) -> String {
    for line in output.lines() {
        if let Some(idx) = line.find("Score: ") {
            return line[idx + 7..].trim().to_string();
        }
    }
    "NOT_FOUND".to_string()
}

/// Provisions the calibration asset (corne.json) so the agent doesn't hang trying to download it.
#[allow(dead_code)]
pub fn setup_calibration_assets(data_root: &std::path::Path) {
    let corne_json = r#"{
        "meta": { "name": "corne", "author": "foostan", "version": "1", "notes": "", "type": "split" },
        "geometry": {
            "keys": [
                {"index":0, "id":"k0", "x":0, "y":0, "w":1, "h":1, "hand":0, "finger":1, "row":0, "col":0},
                {"index":1, "id":"k1", "x":1, "y":0, "w":1, "h":1, "hand":0, "finger":2, "row":0, "col":1}
            ],
            "prime_slots": [0, 1],
            "med_slots": [],
            "low_slots": [],
            "home_row": 0
        },
        "layouts": { "default": "A B" }
    }"#;
    let user_kb_dir = data_root.join("user/keyboards");
    std::fs::create_dir_all(&user_kb_dir).expect("failed to create kb dir");
    std::fs::write(user_kb_dir.join("corne.json"), corne_json).expect("failed to write corne.json");
}
