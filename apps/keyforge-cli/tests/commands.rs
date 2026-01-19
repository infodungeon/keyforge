// apps/keyforge-cli/tests/commands.rs

//! Integration tests for CLI command execution. Verifies the correctness of the `fetch`
//! and `init` commands, ensuring proper workspace initialization, asset synchronization,
//! and hermetic test isolation.

mod common;
use keyforge_testing::HermeticWorkspace;
use serde_json::Value;
use std::fs;
use std::process::Command;

#[test]
fn test_init_workspace() {
    let temp = tempfile::tempdir().unwrap();
    let bin = common::get_binary_path();

    let output = Command::new(&bin)
        .arg("init")
        .arg(temp.path())
        .output()
        .expect("Failed to run init");

    assert!(output.status.success());
    assert!(temp.path().join("data/user/keyboards").exists());
    assert!(temp.path().join("data/user/corpora").exists());
}

#[test]
fn test_list_assets() {
    let ctx = HermeticWorkspace::new();
    let bin = common::get_binary_path();

    let kb_dir = ctx.data_root.join("user/keyboards");
    fs::create_dir_all(&kb_dir).unwrap();

    let json = r#"{
        "meta": { "name": "Test Board", "author": "Unit Test", "version": "1", "type": "ortho" },
        "geometry": { 
            "keys": [{"index":0, "x":0.0, "y":0.0, "hand":0, "finger":1}], 
            "prime_slots": [0],
            "med_slots": [],
            "low_slots": [],
            "home_row": 2 
        },
        "layouts": {}
    }"#;
    fs::write(kb_dir.join("test_kb.json"), json).unwrap();

    // DEBUG: Verify file existence
    println!("DEBUG: Data Root: {:?}", ctx.data_root);
    println!("DEBUG: KB Dir Exists: {}", kb_dir.exists());
    for entry in fs::read_dir(&kb_dir).unwrap() {
        println!("DEBUG: Found file: {:?}", entry.unwrap().path());
    }

    let output = Command::new(&bin)
        .env("KEYFORGE_DATA_DIR", &ctx.data_root)
        .args(["list", "keyboards"])
        .output()
        .expect("Failed to run list");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("STDOUT:\n{}", stdout);
    println!("STDERR:\n{}", stderr);

    assert!(output.status.success(), "Command failed: {}", stderr);
    assert!(
        stdout.contains("test_kb"),
        "Output did not contain 'test_kb'"
    );
}

#[test]
fn test_validate_output() {
    let ctx = HermeticWorkspace::new().with_default_assets();
    let bin = common::get_binary_path();

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
            "--layout",
            "default",
        ])
        .output()
        .expect("Failed to run validate");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        eprintln!("STDOUT:\n{}", stdout);
        eprintln!("STDERR:\n{}", stderr);
    }

    assert!(output.status.success());

    // Verify JSON output
    let json: Value = serde_json::from_str(&stdout).expect("Failed to parse output JSON");
    assert!(
        json.get("score").is_some(),
        "JSON output missing 'score' field: {}",
        stdout
    );
}
