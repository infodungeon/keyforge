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
        let cost_path = dir.path().join("poison_cost.csv");
        let corpus_dir = dir.path().join("poison_corpus");
        let keyboard_path = dir.path().join("poison_keyboard.json");
        let weights_path = dir.path().join("poison_weights.json");
        let keycodes_path = get_real_keycodes_path();

        let key_ids = [
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

        // 1. Poisoned Cost Matrix
        let mut cost_file = File::create(&cost_path).unwrap();
        writeln!(cost_file, "From_Key,To_Key,Cost").unwrap();

        for (i, k1) in key_ids.iter().enumerate() {
            for (j, k2) in key_ids.iter().enumerate() {
                let mut cost = 1.0;
                // Poison Home Row indices (10-19)
                // If BOTH keys are in home row (or same key), penalty is nuclear.
                if (10..=19).contains(&i) || (10..=19).contains(&j) {
                    cost = 1_000_000_000.0;
                }
                writeln!(cost_file, "{},{},{}", k1, k2, cost).unwrap();
            }
        }

        // 2. Corpus
        fs::create_dir(&corpus_dir).unwrap();

        // 1grams: Remove Monogram incentive for 'e'
        let mut f1 = File::create(corpus_dir.join("1grams.csv")).unwrap();
        writeln!(f1, "char,freq").unwrap();
        writeln!(f1, "e,1").unwrap();
        for c in "taoinshrdlu".chars() {
            writeln!(f1, "{},10", c).unwrap();
        }

        // 2grams: Trigger Cost Matrix Lookup
        let mut f2 = File::create(corpus_dir.join("2grams.csv")).unwrap();
        writeln!(f2, "char1,char2,freq").unwrap();
        // Bigram e-e triggers massive poison cost if e is on home row
        writeln!(f2, "e,e,10000").unwrap();

        let mut f3 = File::create(corpus_dir.join("3grams.csv")).unwrap();
        writeln!(f3, "char1,char2,char3,freq").unwrap();

        // 3. Keyboard
        let mut kb_file = File::create(&keyboard_path).unwrap();
        let mut keys_json = Vec::new();
        for r in 0..3 {
            for c in 0..10 {
                let idx = r * 10 + c;
                let id = key_ids.get(idx).unwrap_or(&"Unknown");
                keys_json.push(format!(
                    r#"{{"id": "{}", "hand": {}, "finger": {}, "row": {}, "col": {}, "x": {}, "y": {}}}"#,
                    id, if c < 5 { 0 } else { 1 }, c % 5, r, c, c as f32, r as f32
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

        // FIXED: Inject the calculated slots into the JSON string
        let json = format!(
            r#"{{ "meta": {{ "name": "Poison", "type": "ortho" }}, "geometry": {{ "keys": [{}], "prime_slots": [{}], "med_slots": [{}], "low_slots": [{}], "home_row": 1 }}, "layouts": {{ }} }}"#,
            keys_json.join(","),
            prime,
            med,
            low
        );
        writeln!(kb_file, "{}", json).unwrap();

        // 4. Weights
        let mut w_file = File::create(&weights_path).unwrap();
        writeln!(
            w_file,
            r#"{{ "corpus_scale": 1.0, "weight_finger_effort": 0.0 }}"#
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

#[test]
fn test_poison_pill_constraint() {
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
            "20",
            "--search-steps",
            "5000",
            "--attempts",
            "1",
            "--seed",
            "999",
            "--debug",
            "--tier-high-chars",
            "etaoinshr",
            "--tier-med-chars",
            "ldcumwfgypb",
            "--tier-low-chars",
            "vkjxqz",
        ])
        .output()
        .expect("Failed to run search");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Print logs for debugging visibility
    println!("=== EXECUTION LOGS ===");
    println!("{}", stderr);
    println!("{}", stdout);

    if !output.status.success() {
        panic!("Crash detected.");
    }

    let mut layout_raw = "";

    // Improved Parser: Ignored table headers
    for line in stdout.lines().chain(stderr.lines()) {
        if let Some(idx) = line.find("Layout: ") {
            let content = line[idx + 8..].trim();
            // Filter out table headers like "Layout: OPTIMIZED"
            if !content.contains("OPTIMIZED") && !content.contains("Layout") && content.len() > 10 {
                layout_raw = content;
            }
        }
    }

    if layout_raw.is_empty() {
        panic!("Failed to parse layout from output.");
    }

    let layout = layout_raw.replace(" ", "");
    // Home row is indices 10-19 in a 30-key grid
    let home_row = &layout[10..20];

    println!("\n=== ANALYSIS ===");
    println!("Final Layout: {}", layout_raw);
    println!("Home Row:     {}", home_row);

    if home_row.contains('e') {
        panic!("❌ Poison pill failed! 'e' is still on the home row.");
    } else {
        println!("✅ Poison pill SUCCESS! 'e' was evicted.");
    }
}
