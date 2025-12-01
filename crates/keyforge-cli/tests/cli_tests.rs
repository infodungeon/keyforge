use regex::Regex;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

// NEW: Helper to find the binary relative to the crate
fn get_binary_path() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let mut path = PathBuf::from(manifest_dir);
    // Go up two levels: crates/keyforge-cli -> crates -> workspace root
    path.pop();
    path.pop();
    path.push("target");
    path.push("release");
    path.push("keyforge");
    path
}

struct TestContext {
    _dir: TempDir,
    cost_path: PathBuf,
    ngram_path: PathBuf,
    keyboard_path: PathBuf,
    weights_path: PathBuf,
}

impl TestContext {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let cost_path = dir.path().join("test_cost.csv");
        let ngram_path = dir.path().join("test_ngrams.tsv");
        let keyboard_path = dir.path().join("test_keyboard.json");
        let weights_path = dir.path().join("test_weights.json");

        // 1. Cost Matrix
        let mut cost_file = File::create(&cost_path).unwrap();
        writeln!(cost_file, "From,To,Cost").unwrap();
        let keys = ["KeyQ", "KeyW", "KeyE", "KeyA", "KeyS", "KeyD"];
        for k1 in keys {
            for k2 in keys {
                writeln!(cost_file, "{},{},1000.0", k1, k2).unwrap();
            }
        }
        let filler = ["KeyR", "KeyT", "KeyY", "KeyU", "KeyI", "KeyO", "KeyP"];
        for k in filler {
            writeln!(cost_file, "KeyA,{},1000.0", k).unwrap();
        }

        // 2. N-Grams
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

        // 3. Keyboard Definition
        let mut kb_file = File::create(&keyboard_path).unwrap();
        let mut keys_json = Vec::new();
        for r in 0..3 {
            for c in 0..10 {
                keys_json.push(format!(
                    r#"{{"hand": {}, "finger": 1, "row": {}, "col": {}, "x": {}, "y": {}}}"#,
                    if c < 5 { 0 } else { 1 },
                    r,
                    c,
                    c as f32,
                    r as f32
                ));
            }
        }
        let json = format!(
            r#"{{
                "meta": {{ "name": "TestKB", "author": "Test", "version": "1.0" }},
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

        // 4. Custom Weights File
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
            weights_path,
        }
    }
}

struct TestResult {
    total: f32,
    flow_cost: f32,
    stdout: String,
}

fn strip_ansi(s: &str) -> String {
    let re = Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    re.replace_all(s, "").to_string()
}

fn run_validate(ctx: &TestContext, args: &[&str]) -> TestResult {
    let mut final_args = vec![
        "validate",
        "--layout",
        "qwerty",
        "--debug",
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

    if !output.status.success() {
        eprintln!("STDERR:\n{}", String::from_utf8_lossy(&output.stderr));
        panic!("Binary failed at {:?}", bin_path);
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

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
            if clean_line.trim().is_empty()
                || clean_line.contains("Layout Comparison")
                || clean_line.contains("Bas")
            {
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

    TestResult {
        total,
        flow_cost,
        stdout,
    }
}

const NO_BONUS_ARGS: &[&str] = &[
    "--bonus-inward-roll",
    "0.0",
    "--bonus-bigram-roll-in",
    "0.0",
    "--bonus-bigram-roll-out",
    "0.0",
];

#[test]
fn test_cli_search_execution() {
    let _ = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .status()
        .unwrap();
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
    assert!(output.status.success());
}

#[test]
fn test_cli_flow_metrics() {
    let _ = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .status()
        .unwrap();
    let ctx = TestContext::new();
    let res = run_validate(&ctx, &[]);
    if res.total == 0.0 {
        panic!("Parsing Failed. Total is 0.0\nSTDOUT:\n{}", res.stdout);
    }
    if res.flow_cost == 0.0 {
        panic!(
            "Flow Logic Failure: Flow Cost is 0.0\nSTDOUT:\n{}",
            res.stdout
        );
    }
}

#[test]
fn test_cli_biomechanical_penalties() {
    let _ = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .status()
        .unwrap();
    let ctx = TestContext::new();
    let res_base = run_validate(&ctx, NO_BONUS_ARGS);
    if res_base.total <= 0.0 {
        panic!("Base total <= 0");
    }

    let mut sfb_args = vec!["--penalty-sfb-base", "5000.0"];
    sfb_args.extend_from_slice(NO_BONUS_ARGS);
    let res_sfb = run_validate(&ctx, &sfb_args);

    if res_sfb.total <= res_base.total * 1.1 {
        panic!(
            "SFB Penalty failed. Base: {}, SFB: {}",
            res_base.total, res_sfb.total
        );
    }
}

#[test]
fn test_sanity_check_ranking() {
    let _ = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .status()
        .unwrap();
    let ctx = TestContext::new();
    let res = run_validate(&ctx, NO_BONUS_ARGS);
    if res.total <= 0.0 {
        panic!("Total score <= 0");
    }
}

#[test]
fn test_cli_corpus_scaling() {
    let _ = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .status()
        .unwrap();
    let ctx = TestContext::new();
    let mut args_small = vec!["--corpus-scale", "1000.0"];
    args_small.extend_from_slice(NO_BONUS_ARGS);
    let res_small = run_validate(&ctx, &args_small);
    let mut args_big = vec!["--corpus-scale", "1.0"];
    args_big.extend_from_slice(NO_BONUS_ARGS);
    let res_big = run_validate(&ctx, &args_big);

    if res_big.total <= res_small.total * 10.0 {
        panic!(
            "Corpus scaling failed. Small: {}, Big: {}",
            res_small.total, res_big.total
        );
    }
}

#[test]
fn test_cli_custom_weights_file() {
    let _ = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .status()
        .unwrap();
    let ctx = TestContext::new();

    // 1. Run with default weights
    let res_default = run_validate(&ctx, NO_BONUS_ARGS);

    // 2. Run with custom weights file
    let weights_arg = ctx.weights_path.to_str().unwrap();
    let mut args = vec!["--weights", weights_arg];
    args.extend_from_slice(NO_BONUS_ARGS);

    let res_custom = run_validate(&ctx, &args);

    println!(
        "Default: {}, Custom: {}",
        res_default.total, res_custom.total
    );

    assert!(
        res_custom.total > res_default.total * 2.0,
        "Custom weights file did not significantly increase score."
    );
}
