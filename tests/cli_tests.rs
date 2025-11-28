use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

struct TestContext {
    _dir: TempDir,
    cost_path: PathBuf,
    ngram_path: PathBuf,
}

impl TestContext {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let cost_path = dir.path().join("test_cost.csv");
        let ngram_path = dir.path().join("test_ngrams.tsv");

        // Cost Matrix
        let mut cost_file = File::create(&cost_path).unwrap();
        writeln!(cost_file, "From,To,Cost").unwrap();

        // FIX: Explicitly set high cost for ALL keys involved in our test n-grams
        // to ensure the total score remains positive despite large flow bonuses.
        let keys = ["KeyQ", "KeyW", "KeyE", "KeyA", "KeyS", "KeyD"];

        for k1 in keys {
            for k2 in keys {
                // Set a massive base cost so (Cost - Bonus) is always positive
                writeln!(cost_file, "{},{},1000.0", k1, k2).unwrap();
            }
        }

        // Filler to ensure >10 keys are loaded (required by loader logic)
        let filler = ["KeyR", "KeyT", "KeyY", "KeyU", "KeyI", "KeyO", "KeyP"];
        for k in filler {
            writeln!(cost_file, "KeyA,{},1000.0", k).unwrap();
        }

        // N-Grams
        let mut ngram_file = File::create(&ngram_path).unwrap();

        // SFB (q=Pinky, a=Pinky) -> SFB
        writeln!(ngram_file, "qa\t1000").unwrap();
        writeln!(ngram_file, "we\t1000").unwrap();

        // Flow (Triggers Bonuses)
        writeln!(ngram_file, "asd\t1000").unwrap(); // Inward Roll
        writeln!(ngram_file, "sad\t1000").unwrap(); // Redirect

        // Monograms (Required for valid loading)
        writeln!(ngram_file, "a\t1000").unwrap();
        writeln!(ngram_file, "s\t1000").unwrap();
        writeln!(ngram_file, "d\t1000").unwrap();
        writeln!(ngram_file, "q\t100").unwrap();
        writeln!(ngram_file, "w\t100").unwrap();
        writeln!(ngram_file, "e\t1000").unwrap();

        Self {
            _dir: dir,
            cost_path,
            ngram_path,
        }
    }
}

struct TestResult {
    total: f32,
    flow_cost: f32,
    stdout: String,
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
    ];

    if !args.contains(&"--corpus-scale") {
        final_args.push("--corpus-scale");
        final_args.push("1.0");
    }

    final_args.extend_from_slice(args);

    let output = Command::new("./target/release/keyforge")
        .args(&final_args)
        .output()
        .expect("Failed to execute binary");

    if !output.status.success() {
        eprintln!(
            "Binary STDERR:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        panic!("Binary failed with status: {}", output.status);
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let mut total = 0.0;
    let mut flow_cost = 0.0;

    // PARSING LOGIC
    let mut in_scoring_table = false;

    for line in stdout.lines() {
        if line.contains("SCORING REPORT") {
            in_scoring_table = true;
            continue;
        }
        if line.contains("STATISTICAL ANALYSIS") {
            in_scoring_table = false;
            continue;
        }

        if in_scoring_table && line.contains("qwerty") {
            let parts: Vec<&str> = line.split('|').collect();

            // 1: Total
            if parts.len() > 1 {
                total = parts[1].trim().parse().unwrap_or(0.0);
            }

            // Last column: Flow Cost / NetCost
            if let Some(last_section) = parts.last() {
                // FIX: Removed .trim() before .split_whitespace() (clippy::trim_split_whitespace)
                if let Some(cost_str) = last_section.split_whitespace().last() {
                    flow_cost = cost_str.parse().unwrap_or(0.0);
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

// Constant for disabling all bonuses to ensure positive, scalable scores
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

    let output = Command::new("./target/release/keyforge")
        .args([
            // FIX: Removed '&' borrow (clippy::needless_borrows_for_generic_args)
            "search",
            "--cost",
            ctx.cost_path.to_str().unwrap(),
            "--ngrams",
            ctx.ngram_path.to_str().unwrap(),
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
        .expect("Failed to execute binary");

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
    // For this test, we WANT bonuses to verify they trigger
    let res = run_validate(&ctx, &[]);

    // Flow cost should be non-zero (likely negative due to bonuses)
    if res.flow_cost == 0.0 {
        println!("STDOUT:\n{}", res.stdout);
        panic!("Flow Logic Failure: Flow Cost is 0.0");
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

    // Base run with NO BONUSES
    let res_base = run_validate(&ctx, NO_BONUS_ARGS);

    if res_base.total <= 0.0 {
        println!("STDOUT:\n{}", res_base.stdout);
        panic!("Base total was {} (Expected > 0)", res_base.total);
    }

    // Increase SFB penalty massively
    let mut sfb_args = vec!["--penalty-sfb-base", "5000.0"];
    sfb_args.extend_from_slice(NO_BONUS_ARGS);

    let res_sfb = run_validate(&ctx, &sfb_args);

    if res_sfb.total <= res_base.total * 1.1 {
        println!("Base: {}, SFB: {}", res_base.total, res_sfb.total);
        panic!("SFB Penalty failed to apply significantly");
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

    // Disable bonuses
    let res = run_validate(&ctx, NO_BONUS_ARGS);

    if res.total <= 0.0 {
        println!("STDOUT:\n{}", res.stdout);
        panic!("Total score should be positive, got {}", res.total);
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

    // Scale 1000.0 -> Low Score
    let mut args_small = vec!["--corpus-scale", "1000.0"];
    args_small.extend_from_slice(NO_BONUS_ARGS);
    let res_small = run_validate(&ctx, &args_small);

    // Scale 1.0 -> High Score
    let mut args_big = vec!["--corpus-scale", "1.0"];
    args_big.extend_from_slice(NO_BONUS_ARGS);
    let res_big = run_validate(&ctx, &args_big);

    if res_big.total <= res_small.total * 10.0 {
        println!("Small: {}", res_small.total);
        println!("Big: {}", res_big.total);
        panic!("Corpus scaling failed");
    }
}
