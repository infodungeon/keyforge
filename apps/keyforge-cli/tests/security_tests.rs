use keyforge_testing::HermeticWorkspace;
use std::fs::File;
use std::io::Write;
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
fn test_oversized_keyboard_file() {
    let ctx = HermeticWorkspace::new();
    let kb_path = ctx.data_root.join("user/keyboards/big_kb.json");
    let f = File::create(&kb_path).unwrap();
    f.set_len(101 * 1024 * 1024).unwrap();

    let bin = get_binary_path();
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
    assert!(
        stderr.contains("exceeds size limit"),
        "Stderr should mention size limit: {}",
        stderr
    );
}

#[test]
fn test_deeply_nested_json_keyboard() {
    let ctx = HermeticWorkspace::new();
    let kb_path = ctx.data_root.join("user/keyboards/bomb.json");
    let mut f = File::create(&kb_path).unwrap();

    let mut s = String::from(
        r#"{
        "meta": { "name": "bomb", "type": "ortho" },
        "geometry": { "keys": [], "prime_slots": [], "med_slots": [], "low_slots": [], "home_row": 0 },
        "bomb": "#,
    );

    for _ in 0..500 {
        s.push_str("{\"a\":");
    }
    s.push('1');
    for _ in 0..500 {
        s.push('}');
    }
    s.push('}');

    writeln!(f, "{}", s).unwrap();

    let bin = get_binary_path();
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
        ])
        .output()
        .expect("Failed");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());

    let passed = stderr.contains("recursion limit exceeded") || stderr.contains("Loader Error");
    assert!(
        passed,
        "Stderr should mention recursion or loader error. Got: {}",
        stderr
    );
}

#[test]
fn test_oversized_corpus_file() {
    let ctx = HermeticWorkspace::new();
    let corpus_dir = ctx.data_root.join("user/corpora/big_corpus");
    std::fs::create_dir_all(&corpus_dir).unwrap();
    let f = File::create(corpus_dir.join("1grams.json")).unwrap();
    f.set_len(101 * 1024 * 1024).unwrap();

    let bin = get_binary_path();
    let output = Command::new(&bin)
        .env("KEYFORGE_DATA_DIR", &ctx.data_root)
        .args([
            "validate",
            "--corpus",
            "big_corpus",
            "--keyboard",
            "test_kb",
            "--cost",
            "cost.json",
            "--keycodes",
            "keycodes.json",
        ])
        .output()
        .expect("Failed");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(
        stderr.contains("exceeds size limit"),
        "Stderr should mention size limit: {}",
        stderr
    );
}
