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
    let debug_path = path.join("debug").join("keyforge");
    if debug_path.exists() {
        return debug_path;
    }
    path.join("release").join("keyforge")
}

fn extract_score(output: &str) -> String {
    for line in output.lines() {
        if let Some(idx) = line.find("Score: ") {
            return line[idx + 7..].trim().to_string();
        }
    }
    "NOT_FOUND".to_string()
}

#[test]
fn test_deterministic_output() {
    let ctx = HermeticWorkspace::new().with_default_assets();
    let bin_path = get_binary_path();

    let w_path = ctx.weights_path("default");
    let args = [
        "search",
        "--seed",
        "12345",
        "--search-epochs",
        "5",
        "--attempts",
        "1",
        "--threads",
        "1",
        "--cost",
        "cost.json",
        "--corpus",
        "test_corpus",
        "--keyboard",
        "test_kb",
        "--weights",
        w_path.to_str().unwrap(),
        "--keycodes",
        "keycodes.json",
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

    if !output_a.status.success() {
        panic!("A Failed:\nSTDERR:\n{}", stderr_a);
    }
    if !output_b.status.success() {
        panic!("B Failed:\nSTDERR:\n{}", stderr_b);
    }

    let score_a = extract_score(&stderr_a);
    let score_b = extract_score(&stderr_b);
    let layout_a = stdout_a.trim();
    let layout_b = stdout_b.trim();

    assert_eq!(score_a, score_b, "Score check failed");
    assert_eq!(layout_a, layout_b, "Layout check failed");
    assert_ne!(score_a, "NOT_FOUND");
}
