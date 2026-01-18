// apps/keyforge-cli/tests/search.rs

//! Integration tests for CLI layout search and optimization workflows. Verifies the
//! end-to-end execution of optimization commands, ensuring correct propagation of search
//! parameters, corpus loading, and result validation.

mod common;
use keyforge_testing::HermeticWorkspace;
use std::process::Command;

#[test]
fn test_search_happy_path() {
    let ctx = HermeticWorkspace::new().with_default_assets();
    common::setup_calibration_assets(&ctx.data_root);
    let bin_path = common::get_binary_path();

    let output = Command::new(&bin_path)
        .env("KEYFORGE_DATA_DIR", &ctx.data_root)
        .args([
            "search",
            "--cost", "cost.json",
            "--corpus", "test_corpus",
            "--keyboard", "test_kb",
            "--keycodes", "keycodes.json",
            "--search", "search_epochs=1",
            "--search", "search_steps=10",
            "--time", "5",
        ])
        .output()
        .expect("Failed to run search");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "Search failed");
    assert!(stdout.contains(r#""score":"#), "Output did not contain score");
}

#[test]
fn test_search_determinism() {
    let ctx = HermeticWorkspace::new().with_default_assets();
    common::setup_calibration_assets(&ctx.data_root);
    let bin_path = common::get_binary_path();
    let w_path = ctx.weights_path("default");

    let args = [
        "search",
        "--seed", "12345",
        "--search", "search_epochs=5",
        "--search", "search_steps=10", // Reduced for test speed
        "--attempts", "1",
        "--threads", "1",
        "--cost", "cost.json",
        "--corpus", "test_corpus",
        "--keyboard", "test_kb",
        "--weights", w_path.to_str().unwrap(),
        "--keycodes", "keycodes.json",
    ];

    let output_a = Command::new(&bin_path)
        .env("KEYFORGE_DATA_DIR", &ctx.data_root)
        .env("RUST_LOG", "info")
        .args(args)
        .output()
        .expect("Run A failed");

    let output_b = Command::new(&bin_path)
        .env("KEYFORGE_DATA_DIR", &ctx.data_root)
        .env("RUST_LOG", "info")
        .args(args)
        .output()
        .expect("Run B failed");

    assert!(output_a.status.success());
    assert!(output_b.status.success());

    let stdout_a = String::from_utf8_lossy(&output_a.stdout);
    let stdout_b = String::from_utf8_lossy(&output_b.stdout);

    let json_a: serde_json::Value = serde_json::from_str(&stdout_a).expect("Failed to parse A");
    let json_b: serde_json::Value = serde_json::from_str(&stdout_b).expect("Failed to parse B");

    assert_eq!(json_a["score"], json_b["score"], "Scores diverged");
    assert_eq!(json_a["layout"], json_b["layout"], "Layouts diverged");
}

#[test]
fn test_search_constraints() {
    let ctx = HermeticWorkspace::new().with_poison_pill();
    common::setup_calibration_assets(&ctx.data_root);
    let bin_path = common::get_binary_path();

    let output = Command::new(&bin_path)
        .env("KEYFORGE_DATA_DIR", &ctx.data_root)
        .args([
            "search",
            "--cost", "poison_cost.json",
            "--corpus", "poison_corpus",
            "--keyboard", "poison_keyboard",
            "--weights", ctx.weights_path("poison_weights").to_str().unwrap(),
            "--keycodes", "keycodes.json",
            "--search", "search_epochs=20",
            "--search", "search_steps=50",
            "--attempts", "1",
            "--seed", "999",
            "--tier-high-chars", "etaoinshrdlu",
            "--tier-med-chars", "",
            "--tier-low-chars", "",
        ])
        .output()
        .expect("Failed to run search");

    let json_str = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() || json_str.trim().is_empty() {
        eprintln!("STDOUT:\n{}", json_str);
        eprintln!("STDERR:\n{}", String::from_utf8_lossy(&output.stderr));
    }
    let json: serde_json::Value = serde_json::from_str(&json_str).expect("Failed to parse JSON output");
    let score = json["score"].as_f64().unwrap_or(0.0);

    // If the poison pill worked (constraint respected), the score should be reasonable.
    // If it failed (pill swallowed/ignored), score would be massive due to penalties.
    if score > 1_000_000.0 {
        panic!("Poison pill failed! Score too high: {}", score);
    }
}