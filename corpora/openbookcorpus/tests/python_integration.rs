use std::process::Command;
use std::path::Path;

fn run_script(script_name: &str) {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let script_path = Path::new(manifest_dir)
        .join("tests")
        .join(script_name);

    assert!(script_path.exists(), "Script not found: {:?}", script_path);

    println!("Running {}...", script_name);
    
    let output = Command::new("python3") // Use "python" on Windows
        .arg(script_path)
        .output()
        .expect("Failed to execute Python script");

    if !output.status.success() {
        println!("--- STDOUT ({}) ---", script_name);
        println!("{}", String::from_utf8_lossy(&output.stdout));
        println!("--- STDERR ({}) ---", script_name);
        println!("{}", String::from_utf8_lossy(&output.stderr));
        panic!("{} failed", script_name);
    } else {
        // Optional: Print stdout even on success if you want to see the stats
        println!("{}", String::from_utf8_lossy(&output.stdout));
    }
}

#[test]
fn validate_1grams() {
    run_script("validate_1grams.py");
}

#[test]
fn validate_ngrams() {
    run_script("validate_ngrams.py");
}

#[test]
fn validate_words() {
    run_script("validate_words.py");
}