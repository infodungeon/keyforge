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

pub use keyforge_testing_macros::kf_test;

use keyforge_infra::{initialize_workspace, FsProvider, InitMode};
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

        // Create marker
        std::fs::write(root.join(keyforge_infra::init::WORKSPACE_MARKER), "test\n")
            .expect("failed marker");

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
    ///
    /// # Panics
    ///
    /// Panics if serialization of default assets fails.
    #[must_use]
    #[allow(clippy::too_many_lines, clippy::unwrap_used)]
    pub fn with_default_assets(self) -> Self {
        use keyforge_model::types::{ColIndex, FingerIndex, HandIndex, KeyCode, RowIndex};
        use keyforge_model::{
            cost_model::{CostModel, FingerDefinition, HandDefinition, ModelDefinition, RowCosts},
            geometry::{KeyboardDefinition, KeyboardGeometry, KeyboardMeta},
            keycodes::KeycodeDefinition,
            KeyNode,
        };
        use std::collections::HashMap;

        // 1. Keycodes
        let keycodes = vec![
            KeycodeDefinition {
                code: KeyCode(0),
                id: "KC_NO".into(),
                label: " ".into(),
                aliases: vec!["NO".into()],
            },
            KeycodeDefinition {
                code: KeyCode(1),
                id: "KC_TRNS".into(),
                label: "▽".into(),
                aliases: vec!["TRNS".into()],
            },
            KeycodeDefinition {
                code: KeyCode(97),
                id: "KC_A".into(),
                label: "a".into(),
                aliases: vec!["A".into()],
            },
            KeycodeDefinition {
                code: KeyCode(98),
                id: "KC_B".into(),
                label: "b".into(),
                aliases: vec!["B".into()],
            },
        ];
        let keycodes_json = serde_json::to_string_pretty(&keycodes).unwrap();
        self.write_file("user/config/keycodes.json", &keycodes_json);
        self.write_file("system/config/keycodes.json", &keycodes_json);

        // 2. Cost Matrix
        let mut static_costs = HashMap::new();
        let mut base_costs = RowCosts::new();
        base_costs.insert(RowIndex(0), 100.0);

        let fingers_def = FingerDefinition::Standard(keyforge_model::cost_model::FingerReach {
            base: base_costs,
            inner: HashMap::default(),
            outer: HashMap::default(),
        });

        let mut fingers = HashMap::new();
        fingers.insert(
            "thumb".into(),
            FingerDefinition::Thumb(HashMap::from([("pos_1".into(), 100.0)])),
        );
        fingers.insert("index".into(), fingers_def.clone());
        fingers.insert("middle".into(), fingers_def.clone());
        fingers.insert("ring".into(), fingers_def.clone());
        fingers.insert("pinky".into(), fingers_def);

        static_costs.insert("universal_hand".into(), HandDefinition { fingers });

        let mut models = HashMap::new();
        let model_def = ModelDefinition {
            description: "Test Model".into(),
            static_costs: static_costs.clone(),
        };
        models.insert("model_a_row_staggered".into(), model_def.clone());
        models.insert(
            "model_ortho".into(),
            ModelDefinition {
                description: "Test Ortho".into(),
                static_costs,
            },
        );

        let cost_model = CostModel {
            meta: keyforge_model::cost_model::CostModelMeta {
                version: "2.0".into(),
                description: "Test".into(),
                unit: "pts".into(),
            },
            models,
            dynamic_rules: keyforge_model::cost_model::DynamicRules::default(),
        };
        let cost_json = serde_json::to_string_pretty(&cost_model).unwrap();
        self.write_file("user/weights/cost.json", &cost_json);
        self.write_file("user/weights/default.json", "{}");

        // 3. Keyboard
        let geometry = KeyboardGeometry {
            keys: vec![
                KeyNode {
                    index: 0,
                    x: 0.0,
                    y: 0.0,
                    hand: HandIndex(0),
                    finger: FingerIndex::INDEX,
                    row: RowIndex(0),
                    col: ColIndex(0),
                    ..Default::default()
                },
                KeyNode {
                    index: 1,
                    x: 1.0,
                    y: 0.0,
                    hand: HandIndex(0),
                    finger: FingerIndex::MIDDLE,
                    row: RowIndex(0),
                    col: ColIndex(1),
                    ..Default::default()
                },
            ],
            prime_slots: vec![
                keyforge_model::types::KeyIndex(0),
                keyforge_model::types::KeyIndex(1),
            ],
            med_slots: vec![],
            low_slots: vec![],
            home_row: 0,
        };

        let kb_def = KeyboardDefinition {
            meta: KeyboardMeta {
                name: "Test KB".into(),
                author: "Test".into(),
                version: "1.0".into(),
                kb_type: "ortho".into(),
                notes: String::new(),
            },
            geometry,
            layouts: HashMap::from([("default".into(), "a b".into())]),
        };
        let kb_json = serde_json::to_string_pretty(&kb_def).unwrap();
        self.write_file("user/keyboards/test_kb.json", &kb_json);

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
                },
                "model_ortho": {
                    "description": "Poison Ortho",
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
        self.root
            .join("user/keyboards")
            .join(format!("{name}.json"))
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

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;

    #[test]
    fn test_hermetic_workspace_lifecycle() {
        let ws = HermeticWorkspace::new()
            .with_default_assets()
            .with_poison_pill();

        // Check core files
        assert!(ws.root.exists());
        assert!(ws.keyboard_path("test_kb").exists());
        assert!(ws.weights_path("poison_weights").exists());
        assert!(ws.keycodes_path().exists());

        // Check corpus dir
        assert!(ws
            .root
            .join("user/corpora/test_corpus/1grams.json")
            .exists());
    }

    #[test]
    fn test_hermetic_workspace_path_helpers() {
        let ws = HermeticWorkspace::new();
        let kb_path = ws.keyboard_path("foo");
        assert!(kb_path
            .to_string_lossy()
            .contains("user/keyboards/foo.json"));

        let cost_path = ws.cost_path("bar.json");
        assert!(cost_path
            .to_string_lossy()
            .contains("user/weights/bar.json"));
    }
}
