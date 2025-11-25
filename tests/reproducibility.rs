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
        let cost_path = dir.path().join("repo_cost.csv");
        let ngram_path = dir.path().join("repo_ngrams.tsv");

        // Minimal Cost Matrix
        let mut cost_file = File::create(&cost_path).unwrap();
        writeln!(cost_file, "From,To,Cost").unwrap();
        writeln!(cost_file, "KeyQ,KeyW,1.0").unwrap();

        // Minimal N-Grams
        let mut ngram_file = File::create(&ngram_path).unwrap();

        // FIX: Add Monograms so char_freqs > 0.0
        writeln!(ngram_file, "t\t1000").unwrap();
        writeln!(ngram_file, "h\t1000").unwrap();
        writeln!(ngram_file, "e\t1000").unwrap();
        writeln!(ngram_file, "a\t1000").unwrap();
        writeln!(ngram_file, "n\t1000").unwrap();
        writeln!(ngram_file, "d\t1000").unwrap();

        // FIX: Add Bigrams so bigrams is not empty
        writeln!(ngram_file, "th\t500").unwrap();
        writeln!(ngram_file, "he\t500").unwrap();
        writeln!(ngram_file, "an\t500").unwrap();
        writeln!(ngram_file, "nd\t500").unwrap();

        // Trigrams
        writeln!(ngram_file, "the\t1000").unwrap();
        writeln!(ngram_file, "and\t500").unwrap();

        Self {
            _dir: dir,
            cost_path,
            ngram_path,
        }
    }
}

fn extract_score(output: &str) -> String {
    for line in output.lines() {
        if line.starts_with("Score:") {
            return line.to_string();
        }
    }
    "NOT_FOUND".to_string()
}

#[test]
fn test_deterministic_output() {
    let _ = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .status()
        .unwrap();
    let ctx = TestContext::new();
    let bin = "./target/release/keyforge";

    // Shared args
    let args = [
        "search",
        "--seed",
        "12345",
        "--search-epochs",
        "5",
        "--attempts",
        "1",
        "--cost",
        ctx.cost_path.to_str().unwrap(),
        "--ngrams",
        ctx.ngram_path.to_str().unwrap(),
        "--corpus-scale",
        "1.0",
    ];

    // Run A
    let output_a = Command::new(bin)
        .args(&args)
        .output()
        .expect("Run A failed");
    // Run B
    let output_b = Command::new(bin)
        .args(&args)
        .output()
        .expect("Run B failed");

    let stdout_a = String::from_utf8_lossy(&output_a.stdout);
    let stdout_b = String::from_utf8_lossy(&output_b.stdout);

    // FIX: Print stderr if execution failed (helps debugging if this persists)
    if !output_a.status.success() {
        println!("STDERR A:\n{}", String::from_utf8_lossy(&output_a.stderr));
    }

    let score_a = extract_score(&stdout_a);
    let score_b = extract_score(&stdout_b);

    if score_a != score_b || score_a == "NOT_FOUND" {
        println!("--- RUN A ---\n{}", stdout_a);
        println!("--- RUN B ---\n{}", stdout_b);
    }

    assert_eq!(score_a, score_b, "Determinism check failed");
    assert_ne!(score_a, "NOT_FOUND", "Failed to parse score from output");
}
