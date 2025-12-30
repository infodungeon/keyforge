use keyforge_testing::HermeticWorkspace;
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
    let release = path.join("release").join("keyforge");
    if release.exists() {
        return release;
    }
    let debug = path.join("debug").join("keyforge");
    if debug.exists() {
        return debug;
    }
    path.join("debug").join("keyforge.exe")
}

#[test]
fn test_poison_pill_constraint() {
    let ctx = HermeticWorkspace::new().with_poison_pill();
    let bin_path = get_binary_path();

    let output = Command::new(&bin_path)
        .env("KEYFORGE_DATA_DIR", &ctx.data_root)
        .args([
            "search",
            "--cost",
            "poison_cost.json",
            "--corpus",
            "poison_corpus",
            "--keyboard",
            "poison_keyboard",
            "--weights",
            ctx.weights_path("poison_weights").to_str().unwrap(),
            "--keycodes",
            "keycodes.json",
            "--search-epochs",
            "20",
            "--search-steps",
            "5000",
            "--attempts",
            "1",
            "--seed",
            "999",
            "--tier-high-chars",
            "etaoinshrdlu",
            "--tier-med-chars",
            "",
            "--tier-low-chars",
            "",
        ])
        .output()
        .expect("Failed to run search");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("STDOUT:\n{}", stdout);
    println!("STDERR:\n{}", stderr);

    if !output.status.success() {
        panic!("CLI execution failed");
    }

    let score_line = stderr.lines().find(|l| l.contains("Score: ")).unwrap_or("");
    let score_str = score_line.split("Score: ").nth(1).unwrap_or("0").trim();
    let score = score_str.parse::<f64>().unwrap_or(0.0);

    println!("Final Score: {}", score);
    if score > 1_000_000.0 {
        panic!("Poison pill failed! Score too high: {}", score);
    }
}
