mod common;
use keyforge_testing::HermeticWorkspace;
use std::process::Command;

#[test]
fn test_search_happy_path() {
    let ctx = HermeticWorkspace::new().with_default_assets();
    let bin_path = common::get_binary_path();

    let output = Command::new(&bin_path)
        .env("KEYFORGE_DATA_DIR", &ctx.data_root)
        .args([
            "search",
            "--cost", "cost.json",
            "--corpus", "test_corpus",
            "--keyboard", "test_kb",
            "--keycodes", "keycodes.json",
            "--search-epochs", "1",
            "--search-steps", "10",
        ])
        .output()
        .expect("Failed to run search");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "Search failed: {}", stderr);
    assert!(stderr.contains("FINAL RESULT"));
}

#[test]
fn test_search_determinism() {
    let ctx = HermeticWorkspace::new().with_default_assets();
    let bin_path = common::get_binary_path();
    let w_path = ctx.weights_path("default");

    let args = [
        "search",
        "--seed", "12345",
        "--search-epochs", "5",
        "--search-steps", "100", // Reduced for test speed
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

    let stdout_a = String::from_utf8_lossy(&output_a.stdout);
    let stdout_b = String::from_utf8_lossy(&output_b.stdout);
    let stderr_a = String::from_utf8_lossy(&output_a.stderr);
    let stderr_b = String::from_utf8_lossy(&output_b.stderr);

    assert!(output_a.status.success());
    assert!(output_b.status.success());

    let score_a = common::extract_score(&stderr_a);
    let score_b = common::extract_score(&stderr_b);

    assert_eq!(score_a, score_b, "Scores diverged between runs");
    assert_eq!(stdout_a.trim(), stdout_b.trim(), "Layouts diverged between runs");
}

#[test]
fn test_search_constraints() {
    let ctx = HermeticWorkspace::new().with_poison_pill();
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
            "--search-epochs", "20",
            "--search-steps", "5000",
            "--attempts", "1",
            "--seed", "999",
            "--tier-high-chars", "etaoinshrdlu",
            "--tier-med-chars", "",
            "--tier-low-chars", "",
        ])
        .output()
        .expect("Failed to run search");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success());

    let score_str = common::extract_score(&stderr);
    let score = score_str.parse::<f64>().unwrap_or(0.0);

    // If the poison pill worked (constraint respected), the score should be reasonable.
    // If it failed (pill swallowed/ignored), score would be massive due to penalties.
    if score > 1_000_000.0 {
        panic!("Poison pill failed! Score too high: {}", score);
    }
}