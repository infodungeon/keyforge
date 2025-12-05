use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

// Helper to find the binary relative to the workspace root
fn get_binary_path() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let mut path = PathBuf::from(manifest_dir);
    // crates/keyforge-cli -> crates -> workspace root
    path.pop();
    path.pop();
    path.push("target");
    path.push("release");
    path.push("keyforge");
    
    if !path.exists() {
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
}

impl TestContext {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let cost_path = dir.path().join("poison_cost.csv");
        let ngram_path = dir.path().join("poison_ngrams.tsv");
        let keyboard_path = dir.path().join("poison_keyboard.json");

        // Key Identifiers matching the loop below
        // 30 keys total.
        let key_ids = [
            // Row 0 (0-9)
            "KeyQ", "KeyW", "KeyE", "KeyR", "KeyT", "KeyY", "KeyU", "KeyI", "KeyO", "KeyP",
            // Row 1 (10-19) - The Poison Row
            "KeyA", "KeyS", "KeyD", "KeyF", "KeyG", "KeyH", "KeyJ", "KeyK", "KeyL", "Semicolon",
            // Row 2 (20-29)
            "KeyZ", "KeyX", "KeyC", "KeyV", "KeyB", "KeyN", "KeyM", "Comma", "Period", "Slash"
        ];

        // 1. Poisoned Cost Matrix
        let mut cost_file = File::create(&cost_path).unwrap();
        writeln!(cost_file, "From_Key,To_Key,Cost").unwrap();

        for (i, k1) in key_ids.iter().enumerate() {
            for (j, k2) in key_ids.iter().enumerate() {
                if i == j {
                    continue;
                }
                let mut cost = 1.0;
                // Poison Home Row indices (10 through 19)
                if (10..=19).contains(&i) || (10..=19).contains(&j) {
                    cost = 1000.0; // Increased poison magnitude to ensure eviction
                }
                writeln!(cost_file, "{},{},{}", k1, k2, cost).unwrap();
            }
        }

        // 2. N-Grams (Trap 'e')
        let mut ngram_file = File::create(&ngram_path).unwrap();
        writeln!(ngram_file, "e\t1000").unwrap(); // Increase freq to make it matter
        let common = ["t", "a", "o", "i", "n", "s", "r"];
        for c in common {
            writeln!(ngram_file, "e{}\t100", c).unwrap();
            writeln!(ngram_file, "{}e\t100", c).unwrap();
        }

        // 3. Dummy Keyboard with IDs
        let mut kb_file = File::create(&keyboard_path).unwrap();
        let mut keys_json = Vec::new();
        for r in 0..3 {
            for c in 0..10 {
                let idx = r * 10 + c;
                let id = key_ids.get(idx).unwrap_or(&"Unknown");
                
                keys_json.push(format!(
                    r#"{{"id": "{}", "hand": {}, "finger": {}, "row": {}, "col": {}, "x": {}, "y": {}}}"#,
                    id,
                    if c < 5 { 0 } else { 1 },
                    // Fix finger assignment to 0-4 to avoid builder panic
                    c % 5, 
                    r,
                    c,
                    c as f32,
                    r as f32
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
            r#"{{
                "meta": {{ "name": "PoisonPill", "author": "Test", "version": "1.0" }},
                "geometry": {{
                    "keys": [{}],
                    "prime_slots": [{}],
                    "med_slots": [{}],
                    "low_slots": [{}],
                    "home_row": 1
                }},
                "layouts": {{ }}
            }}"#,
            keys_json.join(","),
            prime,
            med,
            low
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

#[test]
fn test_poison_pill_constraint() {
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
            "50", // Reduced epochs for speed
            "--search-steps",
            "1000",
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

    let mut layout_raw = "";
    for line in stdout.lines() {
        if let Some(idx) = line.find("Layout: ") {
            layout_raw = line[idx + 8..].trim();
            break;
        }
    }

    let layout = layout_raw.replace(" ", "");

    if layout.len() < 30 {
        println!("STDOUT:\n{}", stdout);
        panic!(
            "Invalid layout length or layout not found. Found: '{}' (Len: {})",
            layout,
            layout.len()
        );
    }

    // Home row is indices 10-19
    let home_row = &layout[10..20];

    // 'e' should be evicted from home row due to high cost
    if home_row.contains('e') {
        println!("STDOUT:\n{}", stdout);
        panic!(
            "Poison pill failed! 'e' found in poisoned home row: '{}'. Layout: {}",
            home_row, layout
        );
    }
}