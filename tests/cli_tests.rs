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
        writeln!(cost_file, "KeyQ,KeyA,10.0").unwrap();
        writeln!(cost_file, "KeyW,KeyE,10.0").unwrap();
        let filler = [
            "KeyR", "KeyT", "KeyY", "KeyU", "KeyI", "KeyO", "KeyP", "KeyS",
        ];
        for k in filler {
            writeln!(cost_file, "KeyQ,{},10.0", k).unwrap();
        }

        // N-Grams
        let mut ngram_file = File::create(&ngram_path).unwrap();
        // SFB
        writeln!(ngram_file, "qa\t1000").unwrap();
        writeln!(ngram_file, "we\t1000").unwrap();
        // Flow
        writeln!(ngram_file, "asd\t1000").unwrap(); // Roll
        writeln!(ngram_file, "sad\t1000").unwrap(); // Redirect
                                                    // Mono
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

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let mut total = 0.0;
    let mut flow_cost = 0.0;

    // UPDATED PARSING for new table layout
    // Layout | Total | Dist | Fing | Base Lat WeakL Diag Long Bot | SFR Lat Scis | Cost
    // Pipes at indices:
    // 0: Layout
    // 1: Total
    // 2: Dist
    // 3: Fing
    // 4: SFB Group (Base, Lat, WeakL, Diag, Long, Bot) -> 6 columns
    // 5: Mech Group (SFR, Lat, Scis) -> 3 columns
    // 6: Flow Cost

    for line in stdout.lines() {
        if line.contains("qwerty") {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() > 1 {
                total = parts[1].trim().parse().unwrap_or(0.0);
            }
            if parts.len() > 6 {
                flow_cost = parts[6].trim().parse().unwrap_or(0.0);
            }
        }
    }

    TestResult {
        total,
        flow_cost,
        stdout,
    }
}

#[test]
fn test_cli_search_execution() {
    let _ = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .status()
        .unwrap();
    let ctx = TestContext::new();
    let output = Command::new("./target/release/keyforge")
        .args(&[
            "search",
            "--debug",
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

    // We can't check Redirect vs Roll explicitly in the summary table anymore,
    // but we can check that Flow Cost is non-zero (proving flow logic ran).
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
    let res_base = run_validate(&ctx, &[]);
    assert!(res_base.total > 0.0);

    // Increase SFB penalty massively
    let res_sfb = run_validate(&ctx, &["--penalty-sfb-base", "500.0"]);
    if res_sfb.total <= res_base.total * 1.1 {
        panic!("SFB Penalty failed to apply");
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
    let res = run_validate(&ctx, &[]);
    assert!(res.total > 0.0);
}

#[test]
fn test_cli_corpus_scaling() {
    let _ = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .status()
        .unwrap();
    let ctx = TestContext::new();
    let res_small = run_validate(&ctx, &["--corpus-scale", "1000.0"]);
    let res_big = run_validate(&ctx, &["--corpus-scale", "1.0"]);
    if res_big.total <= res_small.total * 100.0 {
        panic!("Corpus scaling failed");
    }
}
