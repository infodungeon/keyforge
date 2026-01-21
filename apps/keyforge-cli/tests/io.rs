// apps/keyforge-cli/tests/io.rs

//! Integration tests for CLI path resolution and I/O operations. Verifies the robustness
//! of file path expansion, workspace-relative lookups, and defensive I/O error handling
//! across various edge cases.

#![allow(clippy::expect_used, clippy::unwrap_used)]

mod common;
use keyforge_testing::HermeticWorkspace;
use std::fs;
use std::process::Command;

#[test]
fn test_resolve_absolute() {
    let ctx = HermeticWorkspace::new().with_default_assets();
    let bin = common::get_binary_path();

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
            "--layout",
            "default",
        ])
        .output()
        .expect("Failed to execute keyforge");

    if !output.status.success() {
        eprintln!("STDOUT:\n{}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR:\n{}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success(), "Absolute path resolution failed");
}

#[test]
fn test_resolve_cwd() {
    let ctx = HermeticWorkspace::new().with_default_assets();
    let bin = common::get_binary_path();

    let ext_dir = tempfile::tempdir().unwrap();
    let ext_kb = ext_dir.path().join("ext_kb.json");
    fs::copy(ctx.keyboard_path("test_kb"), &ext_kb).unwrap();

    let output = Command::new(&bin)
        .current_dir(ext_dir.path())
        .env("KEYFORGE_DATA_DIR", &ctx.data_root)
        .args([
            "validate",
            "--keyboard",
            "./ext_kb.json",
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
        .expect("Failed to execute keyforge");

    if !output.status.success() {
        eprintln!("STDOUT:\n{}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR:\n{}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success(), "CWD-relative resolution failed");
}

#[test]
fn test_resolve_workspace() {
    let ctx = HermeticWorkspace::new().with_default_assets();
    let bin = common::get_binary_path();

    // "test_kb" is in user/keyboards/test_kb.json inside the hermetic workspace
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
        .expect("Failed to execute keyforge");

    if !output.status.success() {
        eprintln!("STDOUT:\n{}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR:\n{}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(
        output.status.success(),
        "Workspace-relative resolution failed"
    );
}
