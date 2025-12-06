use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

// 1. ROBUST BINARY FINDER
fn get_binary_path() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let mut path = PathBuf::from(manifest_dir);
    if path.ends_with("keyforge-cli") {
        path.pop();
        path.pop();
    }
    path.push("target");

    let release = path.join("release").join("keyforge");
    if release.exists() {
        return release;
    }

    let debug = path.join("debug").join("keyforge");
    if debug.exists() {
        return debug;
    }

    panic!("❌ Binary not found. Run 'cargo build --release' first.");
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
        panic!("keycodes.json not found");
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
        let cost_path = dir.path().join("repo_cost.csv");
        let corpus_dir = dir.path().join("repo_corpus");
        let keyboard_path = dir.path().join("repo_keyboard.json");
        let weights_path = dir.path().join("repo_weights.json");
        let keycodes_path = get_real_keycodes_path();

        // 1. Minimal Cost
        let mut cost_file = File::create(&cost_path).unwrap();
        writeln!(cost_file, "From,To,Cost").unwrap();
        writeln!(cost_file, "KeyQ,KeyW,1000.0").unwrap();
        let filler = [
            "KeyE", "KeyR", "KeyT", "KeyY", "KeyU", "KeyI", "KeyO", "KeyP",
        ];
        for k in filler {
            writeln!(cost_file, "KeyQ,{},1000.0", k).unwrap();
        }

        // 2. Minimal Corpus
        fs::create_dir(&corpus_dir).unwrap();
        let mut f1 = File::create(corpus_dir.join("1grams.csv")).unwrap();
        writeln!(f1, "char,freq").unwrap();
        writeln!(f1, "q,1000").unwrap();
        writeln!(f1, "w,1000").unwrap();

        let mut f2 = File::create(corpus_dir.join("2grams.csv")).unwrap();
        writeln!(f2, "char1,char2,freq").unwrap();
        writeln!(f2, "q,w,1000").unwrap();
        writeln!(f2, "w,q,1000").unwrap();

        let mut f3 = File::create(corpus_dir.join("3grams.csv")).unwrap();
        writeln!(f3, "char1,char2,char3,freq").unwrap();
        writeln!(f3, "q,w,q,1000").unwrap();

        // 3. Minimal 30-key Keyboard
        let mut kb_file = File::create(&keyboard_path).unwrap();
        let mut keys_json = Vec::new();
        for r in 0..3 {
            for c in 0..10 {
                keys_json.push(format!(
                    r#"{{"hand": {}, "finger": 1, "row": {}, "col": {}, "x": {}, "y": {}, "w": 1.0, "h": 1.0}}"#,
                    if c < 5 { 0 } else { 1 }, r, c, c as f32, r as f32
                ));
            }
        }
        let prime = (10..20)
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let med = (0..10).map(|i| i.to_string()).collect::<Vec<_>>().join(",");
        let low = (20..30)
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let json = format!(
            r#"{{ "meta": {{ "name": "Repo", "type": "ortho" }}, "geometry": {{ "keys": [{}], "prime_slots": [{}], "med_slots": [{}], "low_slots": [{}], "home_row": 1 }}, "layouts": {{}} }}"#,
            keys_json.join(","),
            prime,
            med,
            low
        );
        writeln!(kb_file, "{}", json).unwrap();

        // 4. Default Weights
        let mut w_file = File::create(&weights_path).unwrap();
        writeln!(
            w_file,
            r#"{{ "corpus_scale": 1.0, "finger_penalty_scale": "1.0,1.0,1.0,1.0,1.0" }}"#
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

fn extract_score(output: &str) -> String {
    for line in output.lines() {
        if let Some(idx) = line.find("Score: ") {
            return line[idx + 7..].trim().to_string();
        }
    }
    "NOT_FOUND".to_string()
}

#[test]
fn test_deterministic_output() {
    let ctx = TestContext::new();
    let bin_path = get_binary_path();

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
        "--pinned-keys",
        "",
    ];

    let output_a = Command::new(&bin_path)
        .args(args)
        .output()
        .expect("Run A failed");
    let output_b = Command::new(&bin_path)
        .args(args)
        .output()
        .expect("Run B failed");

    let stdout_a = String::from_utf8_lossy(&output_a.stdout);
    let stdout_b = String::from_utf8_lossy(&output_b.stdout);

    if !output_a.status.success() {
        panic!("A Failed:\n{}", String::from_utf8_lossy(&output_a.stderr));
    }
    if !output_b.status.success() {
        panic!("B Failed:\n{}", String::from_utf8_lossy(&output_b.stderr));
    }

    let score_a = extract_score(&stdout_a);
    let score_b = extract_score(&stdout_b);

    if score_a != score_b || score_a == "NOT_FOUND" {
        println!("--- RUN A ---\n{}", stdout_a);
        println!("--- RUN B ---\n{}", stdout_b);
    }

    assert_eq!(score_a, score_b, "Determinism check failed");
    assert_ne!(score_a, "NOT_FOUND");
}
