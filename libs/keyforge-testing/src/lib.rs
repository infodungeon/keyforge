// Copyright (c) 2025 KeyForge Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # KeyForge Testing Utilities
//!
//! Provides a hermetic workspace and asset injection tools for 
//! integration testing across the KeyForge project.

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;
use keyforge_model::geometry::{KeyboardDefinition, KeyboardMeta, KeyboardGeometry, KeyNode};
use keyforge_model::types::{HandIndex, FingerIndex, RowIndex, ColIndex, KeyIndex};
use keyforge_core::loader::{RawCostData, CostEntry};

/// A hermetic, isolated filesystem workspace for integration tests.
///
/// This provides a temporary directory structure that mimics the KeyForge 
/// data layout, allowing tests to inject assets without side effects.
#[derive(Debug)]
pub struct HermeticWorkspace {
    /// The root temporary directory.
    pub temp_dir: TempDir,
    /// The path to the 'data' root within the temporary directory.
    pub data_root: PathBuf,
}

impl Default for HermeticWorkspace {
    fn default() -> Self {
        Self::new()
    }
}

impl HermeticWorkspace {
    /// Creates a new empty hermetic workspace with the standard directory structure.
    pub fn new() -> Self {
        let temp = tempfile::tempdir().expect("Failed to create temp dir");
        let data_root = temp.path().join("data");

        // Create Sandbox Structure (User Overlay)
        fs::create_dir_all(data_root.join("user/keyboards")).unwrap();
        fs::create_dir_all(data_root.join("user/corpora")).unwrap();
        fs::create_dir_all(data_root.join("user/weights")).unwrap();
        fs::create_dir_all(data_root.join("user/config")).unwrap();

        Self {
            temp_dir: temp,
            data_root,
        }
    }

    /// Populates the workspace with a minimal set of default assets.
    pub fn with_default_assets(mut self) -> Self {
        let kb = self.default_kb();
        self = self.with_keyboard("test_kb", kb)
            .with_corpus("test_corpus", "a", 100)
            .with_cost_matrix("cost.json", vec![CostEntry {
                from: "KC_A".to_string(),
                to: "KC_B".to_string(),
                cost: 10.0,
            }])
            .with_keycodes(r#"[
                { "code": 97, "id": "KC_A", "label": "a", "aliases": [] },
                { "code": 98, "id": "KC_B", "label": "b", "aliases": [] }
            ]"#);
        self.ensure_default_weights();
        self
    }

    /// Injects a "poison pill" set of assets designed to fail validation or scoring.
    ///
    /// Useful for testing error handling and robustness.
    pub fn with_poison_pill(self) -> Self {
        let key_ids = [
            "KeyQ", "KeyW", "KeyE", "KeyR", "KeyT", "KeyY", "KeyU", "KeyI", "KeyO", "KeyP",
            "KeyA", "KeyS", "KeyD", "KeyF", "KeyG", "KeyH", "KeyJ", "KeyK", "KeyL", "Semicolon",
            "KeyZ", "KeyX", "KeyC", "KeyV", "KeyB", "KeyN", "KeyM", "Comma", "Period", "Slash",
        ];
        
        // 1. Poison Keyboard
        let mut keys = Vec::new();
        for r in 0..3 {
            for c in 0..10 {
                let idx = r * 10 + c;
                keys.push(KeyNode {
                    index: idx,
                    label: key_ids[idx].to_string(),
                    x: c as f32,
                    y: r as f32,
                    hand: if c < 5 { HandIndex::LEFT } else { HandIndex::RIGHT },
                    finger: FingerIndex((c % 5) as u8),
                    row: RowIndex(r as i8),
                    col: ColIndex(c as i8),
                    ..Default::default()
                });
            }
        }
        
        let kb_def = KeyboardDefinition {
            meta: KeyboardMeta { name: "Poison".to_string(), author: "Test".to_string(), version: "1".to_string(), kb_type: "ortho".to_string(), ..Default::default() },
            geometry: KeyboardGeometry {
                keys,
                prime_slots: (10..20).map(KeyIndex).collect(),
                med_slots: (0..10).map(KeyIndex).collect(),
                low_slots: (20..30).map(KeyIndex).collect(),
                home_row: 1,
            },
            layouts: std::collections::HashMap::new(),
        };
        
        // 2. Poison Costs
        let mut costs = Vec::new();
        for (i, k1) in key_ids.iter().enumerate() {
            for (j, k2) in key_ids.iter().enumerate() {
                let mut cost = 1.0;
                if (10..=19).contains(&i) || (10..=19).contains(&j) {
                    cost = 1_000_000_000.0;
                }
                costs.push(CostEntry { from: k1.to_string(), to: k2.to_string(), cost });
            }
        }
        
        // 3. Poison Corpus
        let corpus_dir = self.corpus_dir("poison_corpus");
        fs::create_dir_all(&corpus_dir).unwrap();
        let mut grams1 = vec![r#"{"char":"e","freq":1}"#.to_string()];
        for c in "taoinshrdlu".chars() {
            grams1.push(format!(r#"{{"char":"{}","freq":10}}"#, c));
        }
        fs::write(corpus_dir.join("1grams.json"), format!("[{}]", grams1.join(","))).unwrap();
        fs::write(corpus_dir.join("2grams.json"), r#"[{"char1":"e","char2":"e","freq":10000}]"#).unwrap();
        fs::write(corpus_dir.join("3grams.json"), "[]").unwrap();
        fs::write(corpus_dir.join("words.json"), "[]").unwrap();

        // 4. Keycodes (Full ASCII + Specials)
        let mut key_defs = Vec::new();
        for b in 32..=126u8 {
            let c = b as char;
            let id = if c.is_alphanumeric() { format!("Key{}", c.to_ascii_uppercase()) } else { format!("KC_{}", b) };
            let code = if c.is_ascii_alphabetic() { c.to_ascii_lowercase() as u8 } else { b };
            key_defs.push(serde_json::json!({ "code": code, "id": id, "label": c.to_string(), "aliases": [] }));
        }
        let specials = [(59, "Semicolon", ";"), (44, "Comma", ","), (46, "Period", "."), (47, "Slash", "/")];
        for (code, id, label) in specials {
            key_defs.push(serde_json::json!({ "code": code, "id": id.to_string(), "label": label.to_string(), "aliases": [] }));
        }

        self.with_keyboard("poison_keyboard", kb_def)
            .with_cost_matrix("poison_cost.json", costs)
            .with_keycodes(&serde_json::to_string(&key_defs).unwrap())
            .with_weights("poison_weights", r#"{ "weight_finger_effort": 0.0 }"#)
    }

    /// Writes a keyboard definition to the workspace.
    pub fn with_keyboard(self, name: &str, def: KeyboardDefinition) -> Self {
        let path = self.keyboard_path(name);
        let f = File::create(&path).unwrap();
        serde_json::to_writer(f, &def).unwrap();
        self
    }

    /// Creates a corpus with a single character frequency entry.
    pub fn with_corpus(self, name: &str, char: &str, freq: u32) -> Self {
        let corpus_dir = self.corpus_dir(name);
        fs::create_dir_all(&corpus_dir).unwrap();
        
        let mut f = File::create(corpus_dir.join("1grams.json")).unwrap();
        writeln!(f, r#"[{{ "char": "{}", "freq": {} }}]"#, char, freq).unwrap();

        File::create(corpus_dir.join("2grams.json")).unwrap().write_all(b"[]").unwrap();
        File::create(corpus_dir.join("3grams.json")).unwrap().write_all(b"[]").unwrap();
        File::create(corpus_dir.join("words.json")).unwrap().write_all(b"[]").unwrap();
        
        self
    }

    /// Writes a cost matrix JSON file to the workspace.
    pub fn with_cost_matrix(self, filename: &str, entries: Vec<CostEntry>) -> Self {
        let path = self.cost_path(filename);
        let data = RawCostData { entries };
        let f = File::create(&path).unwrap();
        serde_json::to_writer(f, &data).unwrap();
        self
    }

    /// Writes the global keycodes registry file.
    pub fn with_keycodes(self, json: &str) -> Self {
        let path = self.keycodes_path();
        fs::write(path, json).unwrap();
        self
    }

    /// Writes a scoring weights file.
    pub fn with_weights(self, name: &str, json: &str) -> Self {
        let path = self.weights_path(name);
        fs::write(path, json).unwrap();
        self
    }

    /// Ensures the default weights file exists, creating it if necessary.
    pub fn ensure_default_weights(&self) {
        let path = self.weights_path("default");
        if !path.exists() {
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
            let mut f = File::create(&path).unwrap();
            writeln!(f, "{}", default_weights_content).unwrap();
        }
    }

    // Path Helpers for Compatibility
    /// Returns the absolute path to a keyboard definition in the workspace.
    pub fn keyboard_path(&self, name: &str) -> PathBuf {
        self.data_root.join(format!("user/keyboards/{}.json", name))
    }
    /// Returns the absolute path to a cost matrix file in the workspace.
    pub fn cost_path(&self, name: &str) -> PathBuf {
        self.data_root.join(format!("user/weights/{}", name))
    }
    /// Returns the absolute path to a weights file in the workspace.
    pub fn weights_path(&self, name: &str) -> PathBuf {
        self.data_root.join(format!("user/weights/{}.json", name))
    }
    /// Returns the path to the global keycodes registry in the workspace.
    pub fn keycodes_path(&self) -> PathBuf {
        self.data_root.join("user/config/keycodes.json")
    }
    /// Returns the directory path for a corpus in the workspace.
    pub fn corpus_dir(&self, name: &str) -> PathBuf {
        self.data_root.join(format!("user/corpora/{}", name))
    }

    fn default_kb(&self) -> KeyboardDefinition {
        KeyboardDefinition {
            meta: KeyboardMeta {
                name: "TestKB".to_string(),
                author: "Test".to_string(),
                version: "1.0".to_string(),
                kb_type: "ortho".to_string(),
                ..Default::default()
            },
            geometry: KeyboardGeometry {
                keys: vec![
                    KeyNode {
                        index: 0,
                        label: "a".to_string(),
                        x: 0.0,
                        y: 0.0,
                        hand: HandIndex::LEFT,
                        finger: FingerIndex::INDEX,
                        row: RowIndex(0),
                        col: ColIndex(0),
                        ..Default::default()
                    },
                    KeyNode {
                        index: 1,
                        label: "b".to_string(),
                        x: 1.0,
                        y: 0.0,
                        hand: HandIndex::LEFT,
                        finger: FingerIndex::MIDDLE,
                        row: RowIndex(0),
                        col: ColIndex(1),
                        ..Default::default()
                    },
                ],
                prime_slots: vec![KeyIndex(0), KeyIndex(1)],
                med_slots: vec![],
                low_slots: vec![],
                home_row: 0,
            },
            layouts: std::collections::HashMap::from([("default".to_string(), "KC_A KC_B".to_string())]),
        }
    }
}
