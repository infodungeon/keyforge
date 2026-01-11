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
    let debug_path = path.join("debug").join("keyforge");
    if debug_path.exists() {
        return debug_path;
    }
    
    let release_path = path.join("release").join("keyforge");
    if release_path.exists() {
        return release_path;
    }
    
    // Windows fallback
    path.join("debug").join("keyforge.exe")
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
        "geometry": {
            "keys": [{"x":0, "y":0}],
            "home_row": [0]
        },
        "layouts": {}
    }"#;
    let user_kb_dir = data_root.join("user/keyboards");
    std::fs::create_dir_all(&user_kb_dir).expect("failed to create kb dir");
    std::fs::write(user_kb_dir.join("corne.json"), corne_json).expect("failed to write corne.json");
}
