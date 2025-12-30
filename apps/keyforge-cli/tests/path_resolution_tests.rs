use keyforge_testing::HermeticWorkspace;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn get_binary_path() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let mut path = PathBuf::from(manifest_dir);
    if path.ends_with("keyforge-cli") {
        path.pop();
        path.pop();
    }
    path.push("target");
    let debug_path = path.join("debug").join("keyforge");
    if debug_path.exists() {
        return debug_path;
    }
    path.join("release").join("keyforge")
}

#[test]
fn test_absolute_path_resolution() {
    let ctx = HermeticWorkspace::new().with_default_assets();
    let bin = get_binary_path();

    // Use absolute paths from HermeticWorkspace
    let output = Command::new(&bin)
        .env("KEYFORGE_DATA_DIR", &ctx.data_root)
        .args([
            "validate",
            "--keyboard",
            ctx.keyboard_path("test_kb").to_str().unwrap(),
            "--cost",
            ctx.cost_path("cost.json").to_str().unwrap(),
            "--corpus",
            "test_corpus",
            "--weights",
            ctx.weights_path("default").to_str().unwrap(),
            "--keycodes",
            ctx.keycodes_path().to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute keyforge");

    if !output.status.success() {
        panic!(
            "Absolute path resolution failed.\nSTDERR: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn test_cwd_relative_path_resolution() {
    let ctx = HermeticWorkspace::new().with_default_assets();
    let bin = get_binary_path();

    // Create external files in a separate temp dir to test CWD relative paths
    let ext_dir = tempfile::tempdir().unwrap();
    let ext_kb = ext_dir.path().join("ext_kb.json");
    fs::copy(ctx.keyboard_path("test_kb"), &ext_kb).unwrap();

    let output = Command::new(&bin)
        .current_dir(ext_dir.path())
        .env("KEYFORGE_DATA_DIR", &ctx.data_root)
        .args([
            "validate",
            "--keyboard",
            "./ext_kb.json", // Relative to CWD
            "--cost",
            "cost.json",
            "--corpus",
            "test_corpus",
            "--keycodes",
            "keycodes.json",
        ])
        .output()
        .expect("Failed to execute keyforge");

    if !output.status.success() {
        panic!(
            "CWD-relative resolution failed.\nSTDERR: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn test_workspace_relative_resolution() {
    let ctx = HermeticWorkspace::new().with_default_assets();
    let bin = get_binary_path();

    // "test_kb" is in user/keyboards/test_kb.json
    let output = Command::new(&bin)
        .env("KEYFORGE_DATA_DIR", &ctx.data_root)
        .args([
            "validate",
            "--keyboard",
            "test_kb",
            "--cost",
            "cost.json",
            "--corpus",
            "test_corpus",
            "--keycodes",
            "keycodes.json",
        ])
        .output()
        .expect("Failed to execute keyforge");

    if !output.status.success() {
        panic!(
            "Workspace-relative resolution failed.\nSTDERR: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
