// ===== keyforge/tests/reproducibility.rs =====
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

struct TestContext {
    _dir: TempDir,
    cost_path: PathBuf,
    ngram_path: PathBuf,
    keyboard_path: PathBuf, // NEW
}

impl TestContext {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let cost_path = dir.path().join("repo_cost.csv");
        let ngram_path = dir.path().join("repo_ngrams.tsv");
        let keyboard_path = dir.path().join("repo_keyboard.json");

        // Minimal Cost
        let mut cost_file = File::create(&cost_path).unwrap();
        writeln!(cost_file, "From,To,Cost").unwrap();
        writeln!(cost_file, "KeyQ,KeyW,1000.0").unwrap();
        let filler = [
            "KeyE", "KeyR", "KeyT", "KeyY", "KeyU", "KeyI", "KeyO", "KeyP",
        ];
        for k in filler {
            writeln!(cost_file, "KeyQ,{},1000.0", k).unwrap();
        }

        // Minimal N-Grams
        let mut ngram_file = File::create(&ngram_path).unwrap();
        writeln!(ngram_file, "q\t1000").unwrap();
        writeln!(ngram_file, "w\t1000").unwrap();
        writeln!(ngram_file, "qw\t1000").unwrap();
        writeln!(ngram_file, "wq\t1000").unwrap();
        writeln!(ngram_file, "qwq\t1000").unwrap();

        // Minimal 30-key Keyboard
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
                "meta": {{ "name": "RepoKB", "author": "Test", "version": "1.0" }},
                "geometry": {{
                    "keys": [{}],
                    "prime_slots": [], "med_slots": [], "low_slots": [],
                    "home_row": 1
                }},
                "layouts": {{}}
            }}"#,
            keys_json.join(",")
        );
        writeln!(kb_file, "{}", json).unwrap();

        Self {
            _dir: dir,
            cost_path,
            ngram_path,
            keyboard_path,
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
        "--keyboard", // NEW
        ctx.keyboard_path.to_str().unwrap(),
        "--corpus-scale",
        "1.0",
    ];

    let output_a = Command::new(bin).args(args).output().expect("Run A failed");
    let output_b = Command::new(bin).args(args).output().expect("Run B failed");

    let stdout_a = String::from_utf8_lossy(&output_a.stdout);
    let stdout_b = String::from_utf8_lossy(&output_b.stdout);

    if !output_a.status.success() {
        println!("STDERR A:\n{}", String::from_utf8_lossy(&output_a.stderr));
        panic!("Run A failed execution");
    }

    let score_a = extract_score(&stdout_a);
    let score_b = extract_score(&stdout_b);

    if score_a != score_b || score_a == "NOT_FOUND" {
        println!("--- RUN A ---\n{}", stdout_a);
        println!("--- RUN B ---\n{}", stdout_b);
    }

    assert_eq!(score_a, score_b, "Determinism check failed: Scores differ");
    assert_ne!(score_a, "NOT_FOUND", "Failed to parse score from output");
}
