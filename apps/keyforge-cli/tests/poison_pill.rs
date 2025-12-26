use serde::Serialize;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

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
    path.join("debug").join("keyforge.exe")
}

#[derive(Serialize)]
struct KeyDef {
    code: u8,
    id: String,
    label: String,
    aliases: Vec<String>,
}

struct TestContext {
    _dir: TempDir,
    data_root: PathBuf,
    weights_path: PathBuf,
}

impl TestContext {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let data_root = dir.path().join("data");

        fs::create_dir_all(data_root.join("keyboards")).unwrap();
        fs::create_dir_all(data_root.join("corpora/poison_corpus")).unwrap();
        fs::create_dir_all(data_root.join("weights")).unwrap();

        // 1. Keycodes (Safe JSON Generation)
        let kc_file = File::create(data_root.join("keycodes.json")).unwrap();
        let mut key_defs = Vec::new();

        // ASCII range
        for b in 32..=126u8 {
            let c = b as char;
            let id = if c.is_alphanumeric() {
                format!("Key{}", c.to_ascii_uppercase())
            } else {
                format!("KC_{}", b)
            };
            key_defs.push(KeyDef {
                code: b,
                id,
                label: c.to_string(),
                aliases: vec![],
            });
        }

        // Explicit specials used in cost matrix
        let specials = [
            (59, "Semicolon", ";"),
            (44, "Comma", ","),
            (46, "Period", "."),
            (47, "Slash", "/"),
        ];

        for (code, id, label) in specials {
            key_defs.push(KeyDef {
                code,
                id: id.to_string(),
                label: label.to_string(),
                aliases: vec![],
            });
        }

        serde_json::to_writer(&kc_file, &key_defs).unwrap();

        // 2. Cost Matrix (JSON format)
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
        let mut cost_entries = Vec::new();
        for (i, k1) in key_ids.iter().enumerate() {
            for (j, k2) in key_ids.iter().enumerate() {
                let mut cost = 1.0;
                // Poison Home Row (Indices 10-19)
                if (10..=19).contains(&i) || (10..=19).contains(&j) {
                    cost = 1_000_000_000.0;
                }
                cost_entries.push(format!(
                    r#"{{"from_key":"{}","to_key":"{}","cost_ms":{},"confidence_samples":10}}"#,
                    k1, k2, cost
                ));
            }
        }
        let mut cost_file =
            File::create(data_root.join("keyboards").join("poison_cost.json")).unwrap();
        writeln!(cost_file, "[{}]", cost_entries.join(",")).unwrap();

        // 3. Corpus (JSON format)
        let corpus_dir = data_root.join("corpora/poison_corpus");
        let mut grams1 = vec![r#"{"char":"e","freq":1}"#.to_string()];
        for c in "taoinshrdlu".chars() {
            grams1.push(format!(r#"{{"char":"{}","freq":10}}"#, c));
        }
        let mut f1 = File::create(corpus_dir.join("1grams.json")).unwrap();
        writeln!(f1, "[{}]", grams1.join(",")).unwrap();

        let mut f2 = File::create(corpus_dir.join("2grams.json")).unwrap();
        writeln!(f2, r#"[{{"char1":"e","char2":"e","freq":10000}}]"#).unwrap();

        File::create(corpus_dir.join("3grams.json"))
            .unwrap()
            .write_all(b"[]")
            .unwrap();

        // 4. Keyboard
        let mut kb_file = File::create(data_root.join("keyboards/poison_keyboard.json")).unwrap();
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
        let json = format!(
            r#"{{ "meta": {{ "name": "Poison", "type": "ortho" }}, "geometry": {{ "keys": [{}], "prime_slots": [{}], "med_slots": [{}], "low_slots": [{}], "home_row": 1 }}, "layouts": {{ }} }}"#,
            keys_json.join(","),
            prime,
            med,
            low
        );
        writeln!(kb_file, "{}", json).unwrap();

        // 5. Weights
        let weights_path = data_root.join("weights/poison_weights.json");
        let mut w_file = File::create(&weights_path).unwrap();
        writeln!(w_file, r#"{{ "weight_finger_effort": 0.0 }}"#).unwrap();

        Self {
            _dir: dir,
            data_root,
            weights_path,
        }
    }
}

#[test]
fn test_poison_pill_constraint() {
    let ctx = TestContext::new();
    let bin_path = get_binary_path();

    let output = Command::new(&bin_path)
        .env("KEYFORGE_DATA_DIR", &ctx.data_root)
        .args([
            "search",
            "--cost",
            "poison_cost.json",
            "--corpus",
            "poison_corpus",
            "--keyboard",
            "poison_keyboard",
            "--weights",
            ctx.weights_path.to_str().unwrap(),
            "--keycodes",
            "keycodes.json",
            "--search-epochs",
            "20",
            "--search-steps",
            "5000",
            "--attempts",
            "1",
            "--seed",
            "999",
            "--tier-high-chars",
            "etaoinshrdlu",
            "--tier-med-chars",
            "",
            "--tier-low-chars",
            "",
        ])
        .output()
        .expect("Failed to run search");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("STDOUT:\n{}", stdout);
    println!("STDERR:\n{}", stderr);

    if !output.status.success() {
        panic!("CLI execution failed");
    }

    let score_line = stderr.lines().find(|l| l.contains("Score: ")).unwrap_or("");
    let score_str = score_line.split("Score: ").nth(1).unwrap_or("0").trim();
    let score = score_str.parse::<f64>().unwrap_or(0.0);

    println!("Final Score: {}", score);
    if score > 1_000_000.0 {
        panic!("Poison pill failed! Score too high: {}", score);
    }
}
