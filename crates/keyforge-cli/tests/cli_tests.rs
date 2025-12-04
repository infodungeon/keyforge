use regex::Regex;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

// Helper to find the binary relative to the crate
fn get_binary_path() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let mut path = PathBuf::from(manifest_dir);
    // Determine if we are in workspace root or crate dir
    if path.ends_with("keyforge-cli") {
        path.pop(); // crates
        path.pop(); // root
    }

    path.push("target");
    path.push("release");
    path.push("keyforge");

    if !path.exists() {
        // Fallback for debug builds if release not found
        path.pop();
        path.push("debug");
        path.push("keyforge");
    }

    path
}

struct TestContext {
    _dir: TempDir,
    cost_path: PathBuf,
    ngram_path: PathBuf,
    keyboard_path: PathBuf,
    _weights_path: PathBuf, // FIXED: Prefixed with underscore
}

impl TestContext {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let cost_path = dir.path().join("test_cost.csv");
        let ngram_path = dir.path().join("test_ngrams.tsv");
        let keyboard_path = dir.path().join("test_keyboard.json");
        let weights_path = dir.path().join("test_weights.json");

        // 1. Cost Matrix (Strict CSV: From,To,Cost)
        let mut cost_file = File::create(&cost_path).unwrap();
        writeln!(cost_file, "From,To,Cost").unwrap();
        let keys = ["KeyQ", "KeyW", "KeyE", "KeyA", "KeyS", "KeyD"];
        for k1 in keys {
            for k2 in keys {
                writeln!(cost_file, "{},{},1000.0", k1, k2).unwrap();
            }
        }
        // Fillers
        let filler = ["KeyR", "KeyT", "KeyY", "KeyU", "KeyI", "KeyO", "KeyP"];
        for k in filler {
            writeln!(cost_file, "KeyA,{},1000.0", k).unwrap();
        }

        // 2. N-Grams (Strict TSV)
        let mut ngram_file = File::create(&ngram_path).unwrap();
        writeln!(ngram_file, "qa\t1000").unwrap();
        writeln!(ngram_file, "we\t1000").unwrap();
        writeln!(ngram_file, "asd\t1000").unwrap();
        writeln!(ngram_file, "sad\t1000").unwrap();
        writeln!(ngram_file, "a\t1000").unwrap();
        writeln!(ngram_file, "s\t1000").unwrap();
        writeln!(ngram_file, "d\t1000").unwrap();
        writeln!(ngram_file, "q\t100").unwrap();
        writeln!(ngram_file, "w\t100").unwrap();
        writeln!(ngram_file, "e\t1000").unwrap();

        // 3. Keyboard Definition (Valid JSON)
        let mut kb_file = File::create(&keyboard_path).unwrap();
        let mut keys_json = Vec::new();
        // Generate enough keys (30) to pass the "Zero Key" stability check
        for r in 0..3 {
            for c in 0..10 {
                keys_json.push(format!(
                    r#"{{"hand": {}, "finger": 1, "row": {}, "col": {}, "x": {}, "y": {}, "w": 1.0, "h": 1.0, "id": "k{}{}"}}"#,
                    if c < 5 { 0 } else { 1 },
                    r,
                    c,
                    c as f32,
                    r as f32,
                    r, c
                ));
            }
        }
        let json = format!(
            r#"{{
                "meta": {{ "name": "TestKB", "author": "Test", "version": "1.0", "type": "ortho" }},
                "geometry": {{
                    "keys": [{}],
                    "prime_slots": [], "med_slots": [], "low_slots": [],
                    "home_row": 1
                }},
                "layouts": {{
                    "qwerty": "QWERTYUIOPASDFGHJKL;ZXCVBNM,./"
                }}
            }}"#,
            keys_json.join(",")
        );
        writeln!(kb_file, "{}", json).unwrap();

        // 4. Custom Weights
        let mut w_file = File::create(&weights_path).unwrap();
        writeln!(
            w_file,
            r#"{{
            "penalty_sfb_base": 10000.0, 
            "penalty_scissor": 5000.0,
            "finger_penalty_scale": "1.0,1.0,1.0,1.0,1.0"
        }}"#
        )
        .unwrap();

        Self {
            _dir: dir,
            cost_path,
            ngram_path,
            keyboard_path,
            _weights_path: weights_path, // FIXED: Prefixed with underscore
        }
    }
}

fn strip_ansi(s: &str) -> String {
    let re = Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    re.replace_all(s, "").to_string()
}

fn run_validate(ctx: &TestContext, args: &[&str]) -> (f32, f32, String) {
    let mut final_args = vec![
        "validate",
        "--layout",
        "qwerty",
        "--cost",
        ctx.cost_path.to_str().unwrap(),
        "--ngrams",
        ctx.ngram_path.to_str().unwrap(),
        "--keyboard",
        ctx.keyboard_path.to_str().unwrap(),
    ];

    if !args.contains(&"--corpus-scale") {
        final_args.push("--corpus-scale");
        final_args.push("1.0");
    }

    final_args.extend_from_slice(args);

    let bin_path = get_binary_path();
    let output = Command::new(&bin_path)
        .args(&final_args)
        .output()
        .expect("Failed to execute binary");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        panic!("Binary failed:\nSTDOUT:\n{}\nSTDERR:\n{}", stdout, stderr);
    }

    let mut total = 0.0;
    let mut flow_cost = 0.0;
    let mut in_scoring_table = false;

    for line in stdout.lines() {
        let clean_line = strip_ansi(line);

        if clean_line.contains("Layout") && clean_line.contains("Total") {
            in_scoring_table = true;
            continue;
        }

        if in_scoring_table {
            if clean_line.trim().is_empty() || clean_line.contains("Layout Comparison") {
                in_scoring_table = false;
                continue;
            }

            if clean_line.to_lowercase().contains("qwerty") {
                let parts: Vec<&str> = clean_line.split('|').collect();
                if parts.len() > 3 {
                    if let Ok(val) = parts[2].trim().replace(',', "").parse() {
                        total = val;
                    }
                    if let Some(last_col) = parts.iter().rev().find(|s| !s.trim().is_empty()) {
                        if let Ok(val) = last_col.trim().replace(',', "").parse() {
                            flow_cost = val;
                        }
                    }
                    break;
                }
            }
        }
    }

    (total, flow_cost, stdout)
}

#[test]
fn test_cli_search_execution() {
    let ctx = TestContext::new();
    let bin_path = get_binary_path();

    let output = Command::new(&bin_path)
        .args([
            "search",
            "--cost",
            ctx.cost_path.to_str().unwrap(),
            "--ngrams",
            ctx.ngram_path.to_str().unwrap(),
            "--keyboard",
            ctx.keyboard_path.to_str().unwrap(),
            "--corpus-scale",
            "1.0",
            "--search-epochs",
            "1",
            "--search-steps",
            "10",
            "--attempts",
            "1",
        ])
        .output()
        .expect("Failed");

    assert!(
        output.status.success(),
        "Search failed. STDERR: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_cli_flow_metrics() {
    let ctx = TestContext::new();
    // FIXED: Renamed flow to _flow to suppress warning
    let (total, _flow, stdout) = run_validate(&ctx, &[]);
    if total == 0.0 {
        panic!("Parsing Failed. Total is 0.0\nSTDOUT:\n{}", stdout);
    }
}
