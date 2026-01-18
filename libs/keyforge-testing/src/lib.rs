// libs/keyforge-testing/src/lib.rs

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

//! # `KeyForge` Testing
//!
//! Hermetic test harness and fixtures for integration testing.
//! This crate provides a `HermeticWorkspace` that creates a temporary
//! directory structure populated with "Golden Data" for testing the
//! full asset loading pipeline.

use keyforge_infra::{FsProvider, initialize_workspace, InitMode};
use std::path::PathBuf;
use tempfile::TempDir;

/// A self-contained, temporary workspace for integration tests.
#[derive(Debug)]
pub struct HermeticWorkspace {
    _temp: TempDir,
    pub root: PathBuf,
    pub data_root: PathBuf, // Alias for compatibility with CLI tests
    pub provider: FsProvider,
}

impl HermeticWorkspace {
    /// Creates a new hermetic workspace with standard directory structure.
    ///
    /// # Panics
    ///
    /// Panics if the temporary directory cannot be created or initial assets cannot be written.
    #[must_use] 
    #[allow(clippy::expect_used)]
    pub fn new() -> Self {
        let temp = tempfile::tempdir().expect("Failed to create temp dir");
        let root = temp.path().to_path_buf();
        
        // Pre-create dummy required assets to pass validation in initialize_workspace
        let sys = root.join("system");
        let assets = [
            ("config/keycodes.json", "[]"),
            ("weights/cost_matrix.json", "{}"),
            ("corpora/text/en_std/1grams.json", "[]"),
        ];

        for (path, content) in assets {
            let p_sys = sys.join(path);
            if let Some(parent) = p_sys.parent() {
                std::fs::create_dir_all(parent).expect("Failed to create system asset dir");
            }
            std::fs::write(&p_sys, content).expect("Failed to write system asset");

            // Also write to user directory for convenience in CLI tests
            let p_user = root.join("user").join(path);
            if let Some(parent) = p_user.parent() {
                std::fs::create_dir_all(parent).expect("Failed to create user asset dir");
            }
            std::fs::write(p_user, content).expect("Failed to write user asset");
        }

        // Initialize structure (will validate assets now)
        initialize_workspace(&root, InitMode::Create).expect("Failed to init workspace");

        let provider = FsProvider::new(root.clone());
        
        Self {
            _temp: temp,
            data_root: root.clone(),
            root,
            provider,
        }
    }

    /// Populates the workspace with standard test assets.
    #[must_use] 
    pub fn with_default_assets(self) -> Self {
        // 1. Keycodes
        let keycodes_json = r#"[
            {"code": 97, "id": "KC_A", "label": "a", "aliases": []},
            {"code": 98, "id": "KC_B", "label": "b", "aliases": []}
        ]"#;
        self.write_file("user/config/keycodes.json", keycodes_json);

        // 2. Cost Matrix
        let cost_json = r#"{
            "meta": { "version": "2.0", "description": "Test", "unit": "pts" },
            "models": {
                "model_a_row_staggered": {
                    "description": "Test Model",
                    "static_costs": {
                        "universal_hand": {
                            "thumb": { "pos_1": 100.0 },
                            "index": { "base": { "r0": 100.0 } },
                            "middle": { "base": { "r0": 100.0 } },
                            "ring": { "base": { "r0": 100.0 } },
                            "pinky": { "base": { "r0": 100.0 } }
                        }
                    }
                }
            },
            "dynamic_rules": { "sequence_modifiers": {}, "penalties": {}, "constraints": {} }
        }"#;
        self.write_file("user/weights/cost.json", cost_json);
        self.write_file("user/weights/default.json", "{}");

        // 3. Keyboard
        let kb_json = r#"{
            "meta": { "name": "Test KB", "author": "Test", "version": "1.0", "type": "ortho" },
            "geometry": {
                "keys": [
                    {"index": 0, "x": 0.0, "y": 0.0, "hand": 0, "finger": 1, "row": 0, "col": 0},
                    {"index": 1, "x": 1.0, "y": 0.0, "hand": 0, "finger": 2, "row": 0, "col": 1}
                ],
                "prime_slots": [0, 1],
                "med_slots": [],
                "low_slots": [],
                "home_row": 0
            },
            "layouts": { "default": "a b" }
        }"#;
        self.write_file("user/keyboards/test_kb.json", kb_json);

        // 4. Corpus
        let corpus_json = r#"[{"s": "a", "f": 100}, {"s": "b", "f": 50}]"#;
        self.write_file("user/corpora/test_corpus/1grams.json", corpus_json);
        self.write_file("user/corpora/test_corpus/2grams.json", "[]");
        self.write_file("user/corpora/test_corpus/3grams.json", "[]");
        self.write_file("user/corpora/test_corpus/words.json", "[]");

        self
    }

    /// Populates the workspace with "poison pill" assets designed to fail if constraints are ignored.
    #[must_use] 
    pub fn with_poison_pill(self) -> Self {
        // Poison Keyboard: 2 keys.
        // Key 0: Cost 0.
        // Key 1: Cost 1,000,000 (Poison).
        let kb_json = r#"{
            "meta": { "name": "Poison KB", "type": "ortho" },
            "geometry": {
                "keys": [
                    {"index": 0, "x": 0.0, "y": 0.0, "hand": 0, "finger": 1, "row": 0, "col": 0},
                    {"index": 1, "x": 1.0, "y": 0.0, "hand": 0, "finger": 2, "row": 0, "col": 1}
                ],
                "prime_slots": [0],
                "med_slots": [],
                "low_slots": [1],
                "home_row": 0
            },
            "layouts": {}
        }"#;
        self.write_file("user/keyboards/poison_keyboard.json", kb_json);

        // Poison Weights: Massive penalty for High-freq char in Low-tier slot.
        let weights_json = r#"{
            "penalty_high_in_low": 1000000.0
        }"#;
        self.write_file("user/weights/poison_weights.json", weights_json);

        // Poison Corpus: 'e' is high freq.
        let corpus_json = r#"[{"s": "e", "f": 1000}]"#;
        self.write_file("user/corpora/poison_corpus/1grams.json", corpus_json);
        self.write_file("user/corpora/poison_corpus/2grams.json", "[]");
        self.write_file("user/corpora/poison_corpus/3grams.json", "[]");
        self.write_file("user/corpora/poison_corpus/words.json", "[]");
        
        // Cost Model
        let cost_json = r#"{
            "meta": { "version": "2.0", "description": "Poison", "unit": "pts" },
            "models": {
                "model_a_row_staggered": {
                    "description": "Poison Model",
                    "static_costs": {
                        "universal_hand": {
                            "thumb": { "pos_1": 0.0 },
                            "index": { "base": { "r0": 0.0 } },
                            "middle": { "base": { "r0": 0.0 } },
                            "ring": { "base": { "r0": 0.0 } },
                            "pinky": { "base": { "r0": 0.0 } }
                        }
                    }
                }
            },
            "dynamic_rules": { "sequence_modifiers": {}, "penalties": {}, "constraints": {} }
        }"#;
        self.write_file("user/weights/poison_cost.json", cost_json);

        self
    }

    /// Writes a file to the workspace relative to the root.
    ///
    /// # Panics
    ///
    /// Panics if the directory cannot be created or the file cannot be written.
    #[allow(clippy::unwrap_used)]
    pub fn write_file(&self, path: &str, content: &str) {
        let target = self.root.join(path);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(target, content).unwrap();
    }

    // --- Path Helpers ---

    #[must_use] 
    pub fn keyboard_path(&self, name: &str) -> PathBuf {
        self.root.join("user/keyboards").join(format!("{name}.json"))
    }

    #[must_use] 
    pub fn cost_path(&self, name: &str) -> PathBuf {
        self.root.join("user/weights").join(name)
    }

    #[must_use] 
    pub fn weights_path(&self, name: &str) -> PathBuf {
        self.root.join("user/weights").join(format!("{name}.json"))
    }

    #[must_use] 
    pub fn keycodes_path(&self) -> PathBuf {
        self.root.join("user/config/keycodes.json")
    }
}

impl Default for HermeticWorkspace {
    fn default() -> Self {
        Self::new()
    }
}

// Re-export for convenience
pub use keyforge_model::constants;
