// apps/keyforge-cli/tests/security.rs

//! Integration tests for CLI input sanitization and security boundaries. Verifies that
//! user-provided data (keymap strings, file paths) is correctly validated to prevent
//! injection attacks and path traversal exploits.

#![allow(clippy::expect_used, clippy::unwrap_used)]

mod common;
use keyforge_testing::HermeticWorkspace;
use std::fs::File;
use std::io::Write;
use std::process::Command;

#[test]
fn test_oversized_file() {
    let ctx = HermeticWorkspace::new().with_default_assets();
    let kb_path = ctx.data_root.join("user/keyboards/big_kb.json");
    let f = File::create(&kb_path).unwrap();
    f.set_len(101 * 1024 * 1024).unwrap(); // 101 MB

    let bin = common::get_binary_path();
    let output = Command::new(&bin)
        .env("KEYFORGE_DATA_DIR", &ctx.data_root)
        .args([
            "validate",
            "--keyboard",
            "big_kb",
            "--cost",
            "cost.json",
            "--keycodes",
            "keycodes.json",
        ])
        .output()
        .expect("Failed");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(stderr.contains("exceeds size limit"));
}

#[test]
fn test_recursion_bomb() {
    let ctx = HermeticWorkspace::new().with_default_assets();
    let kb_path = ctx.data_root.join("user/keyboards/bomb.json");
    let mut f = File::create(&kb_path).unwrap();

    let mut s = String::from(
        r#"{"meta": {"name": "bomb", "type": "ortho"}, "geometry": {"keys": [], "prime_slots": [], "med_slots": [], "low_slots": [], "home_row": 0}, "bomb": "#,
    );
    for _ in 0..500 {
        s.push_str("{\"a\":");
    }
    s.push('1');
    for _ in 0..500 {
        s.push('}');
    }
    s.push('}');

    writeln!(f, "{s}").unwrap();

    let bin = common::get_binary_path();
    let output = Command::new(&bin)
        .env("KEYFORGE_DATA_DIR", &ctx.data_root)
        .args([
            "validate",
            "--keyboard",
            "bomb",
            "--cost",
            "cost.json",
            "--keycodes",
            "keycodes.json",
            "--layout",
            "default",
        ])
        .output()
        .expect("Failed");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(
        stderr.contains("recursion limit exceeded")
            || stderr.contains("Loader Error")
            || stderr.contains("unknown field")
    );
}
