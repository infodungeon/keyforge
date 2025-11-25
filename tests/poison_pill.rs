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
        let cost_path = dir.path().join("poison_cost.csv");
        let ngram_path = dir.path().join("poison_ngrams.tsv");

        // 1. Poisoned Cost Matrix
        // We assign a cost of 100.0 to any bigram involving the Home Row (indices 10-19).
        // Standard cost is 1.0.
        // Freq will also be 100.0. Total Penalty = 10,000 per occurrence.
        // Normal keys have score ~20. This is a 500x penalty.
        let mut cost_file = File::create(&cost_path).unwrap();
        writeln!(cost_file, "From_Key,To_Key,Cost").unwrap();

        let keys = [
            "KeyQ",
            "KeyW",
            "KeyE",
            "KeyR",
            "KeyT",
            "KeyY",
            "KeyU",
            "KeyI",
            "KeyO",
            "KeyP",
            "KeyA",
            "KeyS",
            "KeyD",
            "KeyF",
            "KeyG",
            "KeyH",
            "KeyJ",
            "KeyK",
            "KeyL",
            "Semicolon",
            "KeyZ",
            "KeyX",
            "KeyC",
            "KeyV",
            "KeyB",
            "KeyN",
            "KeyM",
            "Comma",
            "Period",
            "Slash",
        ];

        for (i, k1) in keys.iter().enumerate() {
            for (j, k2) in keys.iter().enumerate() {
                if i == j {
                    continue;
                }
                let mut cost = 1.0;
                // Poison Home Row indices (10 through 19)
                if (10..=19).contains(&i) || (10..=19).contains(&j) {
                    cost = 100.0;
                }
                writeln!(cost_file, "{},{},{}", k1, k2, cost).unwrap();
            }
        }

        // 2. N-Grams
        let mut ngram_file = File::create(&ngram_path).unwrap();

        // Monogram for Tier sorting
        writeln!(ngram_file, "e\t100").unwrap();

        // Bigrams for Cost Matrix calculation
        let common = ["t", "a", "o", "i", "n", "s", "r"];
        for c in common {
            writeln!(ngram_file, "e{}\t100", c).unwrap();
            writeln!(ngram_file, "{}e\t100", c).unwrap();
        }

        Self {
            _dir: dir,
            cost_path,
            ngram_path,
        }
    }
}

#[test]
fn test_poison_pill_constraint() {
    // 1. Build
    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .status()
        .unwrap();
    assert!(status.success());

    let ctx = TestContext::new();

    // 2. Run Search
    let output = Command::new("./target/release/keyforge")
        .args(&[
            "search",
            "--cost",
            ctx.cost_path.to_str().unwrap(),
            "--ngrams",
            ctx.ngram_path.to_str().unwrap(),
            "--corpus-scale",
            "1.0",
            "--search-epochs",
            "50",
            "--search-steps",
            "2000",
            "--attempts",
            "1",
            "--seed",
            "999",
            "--debug",
        ])
        .output()
        .expect("Failed to run search");

    let stdout = String::from_utf8_lossy(&output.stdout);

    if !output.status.success() {
        println!("STDERR:\n{}", String::from_utf8_lossy(&output.stderr));
        panic!("Keyforge binary crashed");
    }

    // 3. Parse Result to find the Layout string
    let mut layout = "";
    for line in stdout.lines() {
        if line.starts_with("Layout:") {
            layout = line.split_once(": ").unwrap().1.trim();
            break;
        }
    }

    if layout.len() != 30 {
        println!("STDOUT:\n{}", stdout);
        panic!("Invalid layout output or layout not found in stdout");
    }

    // 4. Assert 'e' is evicted from Home Row (indices 10-19)
    let home_row = &layout[10..20];

    if home_row.contains('e') {
        println!("STDOUT:\n{}", stdout);
        panic!(
            "Poison pill failed! 'e' found in poisoned home row: '{}'. Layout: {}",
            home_row, layout
        );
    }
}
