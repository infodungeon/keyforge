use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

pub struct HermeticWorkspace {
    pub temp_dir: TempDir,
    pub data_root: PathBuf,
    pub keyboard_path: PathBuf,
    pub cost_path: PathBuf,
    pub weights_path: PathBuf,
    pub keycodes_path: PathBuf,
    pub corpus_dir: PathBuf,
}

impl Default for HermeticWorkspace {
    fn default() -> Self {
        Self::new()
    }
}

impl HermeticWorkspace {
    pub fn new() -> Self {
        let temp = tempfile::tempdir().expect("Failed to create temp dir");
        let data_root = temp.path().join("data");

        // Create Sandbox Structure (User Overlay)
        fs::create_dir_all(data_root.join("user/keyboards")).unwrap();
        fs::create_dir_all(data_root.join("user/corpora/test_corpus")).unwrap();
        fs::create_dir_all(data_root.join("user/weights")).unwrap();
        fs::create_dir_all(data_root.join("user/config")).unwrap();

        // 1. Cost Matrix (In user/weights)
        let cost_path = data_root.join("user/weights/cost.json");
        let mut f = File::create(&cost_path).unwrap();
        writeln!(f, r#"[{{ "from_key": "KC_A", "to_key": "KC_B", "cost_ms": 10.0, "confidence_samples": 10 }}]"#).unwrap();

        // 2. Corpus (In user/corpora)
        let corpus_dir = data_root.join("user/corpora/test_corpus");
        let mut f = File::create(corpus_dir.join("1grams.json")).unwrap();
        writeln!(
            f,
            r#"[{{ "char": "a", "freq": 100 }}, {{ "char": "b", "freq": 110 }}]"#
        )
        .unwrap();

        let mut f = File::create(corpus_dir.join("2grams.json")).unwrap();
        writeln!(f, r#"[{{ "char1": "a", "char2": "b", "freq": 50 }}]"#).unwrap();

        File::create(corpus_dir.join("3grams.json"))
            .unwrap()
            .write_all(b"[]")
            .unwrap();
        File::create(corpus_dir.join("words.json"))
            .unwrap()
            .write_all(b"[]")
            .unwrap();

        // 3. Keyboard (In user/keyboards)
        let keyboard_path = data_root.join("user/keyboards/test_kb.json");
        let mut f = File::create(&keyboard_path).unwrap();
        let kb_json = r#"{
            "meta": { "name": "TestKB", "author": "Test", "version": "1.0", "type": "ortho" },
            "geometry": {
                "keys": [{"id": "KC_A", "x":0, "y":0, "hand":0, "finger":1, "row":0, "col":0}, {"id": "KC_B", "x":1, "y":0, "hand":0, "finger":2, "row":0, "col":1}],
                "prime_slots": [0, 1], "med_slots": [], "low_slots": [], "home_row": 0
            },
            "layouts": { "default": "KC_A KC_B" }
        }"#;
        writeln!(f, "{}", kb_json).unwrap();

        // 4. Keycodes (In user/config)
        // CHANGED: Use 97 (a) and 98 (b) to match Corpus lowercase standard
        let keycodes_path = data_root.join("user/config/keycodes.json");
        let mut f = File::create(&keycodes_path).unwrap();
        writeln!(
            f,
            r#"[
            {{ "code": 97, "id": "KC_A", "label": "a", "aliases": [] }},
            {{ "code": 98, "id": "KC_B", "label": "b", "aliases": [] }}
        ]"#
        )
        .unwrap();

        let weights_path = data_root.join("user/weights/default.json");

        Self {
            temp_dir: temp,
            data_root,
            keyboard_path,
            cost_path,
            weights_path,
            keycodes_path,
            corpus_dir,
        }
    }

    pub fn ensure_default_weights(&self) {
        if !self.weights_path.exists() {
            let default_weights_content = r#"{
                "penalty_sfb_base": 400.0,
                "penalty_scissor": 25.0,
                "weight_vertical_travel": 1.0,
                "weight_lateral_travel": 3.5,
                "finger_penalty_scale": "0.0,1.0,1.1,1.3,1.6",
                "comfortable_scissors": "21,23,34",
                "loader_trigram_limit": 20000,
                "threshold_sfb_long_row_diff": 2,
                "threshold_scissor_row_diff": 2
            }"#;
            let mut f = File::create(&self.weights_path).unwrap();
            writeln!(f, "{}", default_weights_content).unwrap();
        }
    }
}
