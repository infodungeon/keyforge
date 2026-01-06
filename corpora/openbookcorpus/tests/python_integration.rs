use std::process::Command;
use std::path::{Path, PathBuf};

fn get_data_dir() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .expect("Could not find parent directory of crate")
        .parent()
        .expect("Could not find workspace root")
        .join("corpora_data")
}

fn run_script(script_name: &str, json_filename: &str) {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let script_path = Path::new(manifest_dir)
        .join("tests")
        .join(script_name);

    let data_dir = get_data_dir();
    let json_path = data_dir.join(json_filename);

    assert!(script_path.exists(), "Script not found: {:?}", script_path);
    assert!(json_path.exists(), "JSON output not found: {:?}. Did you run the main program?", json_path);

    println!("Running {} on {}...", script_name, json_path.display());
    
    let output = Command::new("python3")
        .arg(script_path)
        .arg(json_path)
        .output()
        .expect("Failed to execute Python script");

    if !output.status.success() {
        println!("--- STDOUT ({}) ---", script_name);
        println!("{}", String::from_utf8_lossy(&output.stdout));
        println!("--- STDERR ({}) ---", script_name);
        println!("{}", String::from_utf8_lossy(&output.stderr));
        panic!("{} failed", script_name);
    } else {
        println!("{}", String::from_utf8_lossy(&output.stdout));
    }
}

#[test]
fn validate_1grams() {
    run_script("validate_1grams.py", "1grams.json");
}

#[test]
fn validate_ngrams() {
    let data_dir = get_data_dir();
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let script_path = Path::new(manifest_dir).join("tests").join("validate_ngrams.py");

    println!("Running validate_ngrams.py on directory {}...", data_dir.display());

    let output = Command::new("python3")
        .arg(script_path)
        .arg(data_dir)
        .output()
        .expect("Failed to execute Python script");

    if !output.status.success() {
        println!("--- STDOUT ---");
        println!("{}", String::from_utf8_lossy(&output.stdout));
        println!("--- STDERR ---");
        println!("{}", String::from_utf8_lossy(&output.stderr));
        panic!("validate_ngrams failed");
    } else {
        println!("{}", String::from_utf8_lossy(&output.stdout));
    }
}

#[test]
fn validate_words() {
    run_script("validate_words.py", "words.json");
}

#[test]
fn validate_vocabulary() {
    run_script("validate_vocabulary.py", "words.json");
}