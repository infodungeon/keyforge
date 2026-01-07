mod common;
use keyforge_testing::HermeticWorkspace;
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
    let ctx = HermeticWorkspace::new().with_default_assets();
    let bin = common::get_binary_path();

    let output = Command::new(&bin)
        .env("KEYFORGE_DATA_DIR", &ctx.data_root)
        .args(["list", "keyboards"])
        .output()
        .expect("Failed to run list");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("test_kb"));
}

#[test]
fn test_validate_output() {
    let ctx = HermeticWorkspace::new().with_default_assets();
    let bin = common::get_binary_path();

    let output = Command::new(&bin)
        .env("KEYFORGE_DATA_DIR", &ctx.data_root)
        .args([
            "validate",
            "--keyboard", "test_kb",
            "--cost", "cost.json",
            "--corpus", "test_corpus",
            "--keycodes", "keycodes.json",
        ])
        .output()
        .expect("Failed to run validate");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success());
    assert!(stderr.contains("Analysis Report"));
    assert!(stderr.contains("Score:"));
}