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
    let release_path = path.join("release").join("keyforge");
    if release_path.exists() {
        return release_path;
    }
    path.join("debug").join("keyforge.exe")
}

#[test]
fn test_hermetic_search() {
    let ctx = HermeticWorkspace::new().with_default_assets();
    let bin_path = get_binary_path();

    let output = Command::new(&bin_path)
        .env("KEYFORGE_DATA_DIR", &ctx.data_root)
        .args([
            "search",
            "--cost",
            "cost.json", // FsProvider looks in user/weights/
            "--corpus",
            "test_corpus", // FsProvider looks in user/corpora/
            "--keyboard",
            "test_kb", // FsProvider looks in user/keyboards/
            "--keycodes",
            "keycodes.json", // FsProvider looks in user/config/
            "--search-epochs",
            "1",
            "--search-steps",
            "10",
        ])
        .output()
        .expect("Failed to run search");

    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        panic!("Search failed. STDERR: {}", stderr);
    }
    assert!(stderr.contains("FINAL RESULT"));
}
