use regex::Regex;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

// Helper to find the binary, prioritizing release builds
fn get_binary_path() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let mut path = PathBuf::from(manifest_dir);

    if path.ends_with("keyforge-cli") {
        path.pop(); // crates
        path.pop(); // root
    }

    path.push("target");

    // Check Release first
    let release_path = path.join("release").join("keyforge");
    let debug_path = path.join("debug").join("keyforge");

    if release_path.exists() {
        return release_path;
    }
    if debug_path.exists() {
        return debug_path;
    }

    panic!(
        "❌ Test Binary Not Found.\n   Checked:\n   - {:?}\n   - {:?}\n   👉 Run 'cargo build --release' first.",
        release_path, debug_path
    );
}

fn get_real_keycodes_path() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let mut path = PathBuf::from(manifest_dir);
    if path.ends_with("keyforge-cli") {
        path.pop();
        path.pop();
    }
    path.push("data");
    path.push("keycodes.json");

    if !path.exists() {
        panic!(
            "❌ Test Setup Failed: Could not locate real keycodes.json at {:?}",
            path
        );
    }
    path
}

struct TestContext {
    _dir: TempDir,
    cost_path: PathBuf,
    corpus_dir: PathBuf,
    keyboard_path: PathBuf,
    weights_path: PathBuf,
    keycodes_path: PathBuf,
}

impl TestContext {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let cost_path = dir.path().join("test_cost.csv");
        let corpus_dir = dir.path().join("test_corpus");
        let keyboard_path = dir.path().join("test_keyboard.json");
        let weights_path = dir.path().join("test_weights.json");
        let keycodes_path = get_real_keycodes_path();

        // 1. Cost Matrix (Strict CSV)
        let mut cost_file = File::create(&cost_path).unwrap();
        writeln!(cost_file, "From,To,Cost").unwrap();
        let keys = [
            "KeyQ", "KeyW", "KeyE", "KeyR", "KeyT", "KeyY", "KeyU", "KeyI", "KeyO", "KeyP",
        ];
        for k1 in keys {
            for k2 in keys {
                writeln!(cost_file, "{},{},100.0", k1, k2).unwrap();
            }
        }

        // 2. Corpus Bundle
        fs::create_dir(&corpus_dir).unwrap();
        let mut f1 = File::create(corpus_dir.join("1grams.csv")).unwrap();
        writeln!(f1, "char,freq").unwrap();
        writeln!(f1, "q,1000").unwrap();
        writeln!(f1, "w,1000").unwrap();
        writeln!(f1, "e,1000").unwrap();

        let mut f2 = File::create(corpus_dir.join("2grams.csv")).unwrap();
        writeln!(f2, "char1,char2,freq").unwrap();
        writeln!(f2, "q,w,5000").unwrap();
        writeln!(f2, "w,e,5000").unwrap();

        let mut f3 = File::create(corpus_dir.join("3grams.csv")).unwrap();
        writeln!(f3, "char1,char2,char3,freq").unwrap();

        // 3. Keyboard
        let mut kb_file = File::create(&keyboard_path).unwrap();
        let mut keys_json = Vec::new();
        let row_chars = [
            ["Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P"],
            ["A", "S", "D", "F", "G", "H", "J", "K", "L", "SCLN"],
            ["Z", "X", "C", "V", "B", "N", "M", "COMM", "DOT", "SLSH"],
        ];

        for (r, row) in row_chars.iter().enumerate() {
            for (c, char_code) in row.iter().enumerate() {
                let id = format!("Key{}", char_code);
                let finger = match c {
                    0..=4 => 4 - c,
                    5..=9 => c - 5,
                    _ => 1,
                };
                let hand = if c < 5 { 0 } else { 1 };

                keys_json.push(format!(
                    r#"{{"id": "{}", "hand": {}, "finger": {}, "row": {}, "col": {}, "x": {}, "y": {}, "w": 1.0, "h": 1.0}}"#,
                    id, hand, finger, r, c, c as f32, r as f32
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
                    "qwerty": "KC_Q KC_W KC_E KC_R KC_T KC_Y KC_U KC_I KC_O KC_P KC_A KC_S KC_D KC_F KC_G KC_H KC_J KC_K KC_L KC_SCLN KC_Z KC_X KC_C KC_V KC_B KC_N KC_M KC_COMM KC_DOT KC_SLSH"
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
            "penalty_sfb_base": 1000.0,
            "penalty_lateral": 100.0,
            "finger_penalty_scale": "1.0,1.0,1.0,1.0,1.0",
            "corpus_scale": 1.0,
            "bonus_bigram_roll_in": 35.0,
            "bonus_bigram_roll_out": 25.0
        }}"#
        )
        .unwrap();

        Self {
            _dir: dir,
            cost_path,
            corpus_dir,
            keyboard_path,
            weights_path,
            keycodes_path,
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
        "--corpus",
        ctx.corpus_dir.to_str().unwrap(),
        "--keyboard",
        ctx.keyboard_path.to_str().unwrap(),
        "--weights",
        ctx.weights_path.to_str().unwrap(),
        "--keycodes",
        ctx.keycodes_path.to_str().unwrap(),
    ];

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
            "--corpus",
            ctx.corpus_dir.to_str().unwrap(),
            "--keyboard",
            ctx.keyboard_path.to_str().unwrap(),
            "--weights",
            ctx.weights_path.to_str().unwrap(),
            "--keycodes",
            ctx.keycodes_path.to_str().unwrap(),
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
    let (total, _flow, stdout) = run_validate(&ctx, &[]);
    if total == 0.0 {
        panic!("Parsing Failed. Total is 0.0\nSTDOUT:\n{}", stdout);
    }
}
