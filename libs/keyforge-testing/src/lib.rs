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

use keyforge_boundary::SafePath;
use keyforge_infra::fs::init::{initialize_workspace_async, InitMode};
use keyforge_infra::FsProvider;
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
    /// # Errors
    /// Returns `anyhow::Result` if the temporary directory cannot be created or initial assets cannot be written.
    pub async fn new() -> anyhow::Result<Self> {
        let temp = tempfile::tempdir()?;
        let root = temp.path().to_path_buf();

        // Pre-create dummy required assets to pass validation in initialize_workspace
        let sys = root.join("system");
        let assets = [
            ("config/keycodes.json", "[]"),
            ("weights/cost_matrix.json", "{{}}"),
            ("corpora/text/en_std/1grams.json", "[]"),
        ];

        for (path, content) in assets {
            let p_sys = sys.join(path);
            if let Some(parent) = p_sys.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::write(&p_sys, content).await?;

            // Also write to user directory for convenience in CLI tests
            let p_user = root.join("user").join(path);
            if let Some(parent) = p_user.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::write(p_user, content).await?;
        }

        // Create marker
        tokio::fs::write(
            root.join(keyforge_infra::fs::init::WORKSPACE_MARKER),
            "test\n",
        )
        .await?;

        // Initialize structure (will validate assets now)

        let safe_root = SafePath::from_trusted_root_path(root.clone());
        initialize_workspace_async(&safe_root, InitMode::Create).await?;

        let provider = FsProvider::new(safe_root);

        Ok(Self {
            _temp: temp,
            data_root: root.clone(),
            root,
            provider,
        })
    }

    /// Populates the workspace with standard test assets.
    ///
    /// # Errors
    /// Returns `anyhow::Result` if serialization of default assets fails or IO error occurs.
    #[allow(clippy::too_many_lines)]
    pub async fn with_default_assets(self) -> anyhow::Result<Self> {
        use keyforge_model::types::{
            ColIndex, FingerIndex, HandIndex, KeyCode, RowIndex, SpatialUnit,
        };
        use keyforge_model::{
            cost_model::{CostModel, FingerDefinition, HandDefinition, ModelDefinition, RowCosts},
            geometry::{KeyboardDefinition, KeyboardGeometry, KeyboardMeta},
            keycodes::KeycodeDefinition,
            Corpus, KeyNode,
        };
        use std::collections::HashMap;
        use std::sync::Arc;

        // 1. Keycodes
        let keycodes = vec![
            KeycodeDefinition {
                code: KeyCode::new(0),
                id: "KC_NO".into(),
                label: " ".into(),
                aliases: vec!["NO".into()],
            },
            KeycodeDefinition {
                code: KeyCode::new(1),
                id: "KC_TRNS".into(),
                label: "▽".into(),
                aliases: vec!["TRNS".into()],
            },
            KeycodeDefinition {
                code: KeyCode::new(97),
                id: "KC_A".into(),
                label: "a".into(),
                aliases: vec!["A".into()],
            },
            KeycodeDefinition {
                code: KeyCode::new(98),
                id: "KC_B".into(),
                label: "b".into(),
                aliases: vec!["B".into()],
            },
            KeycodeDefinition {
                code: KeyCode::new(116),
                id: "KC_T".into(),
                label: "t".into(),
                aliases: vec!["T".into()],
            },
            KeycodeDefinition {
                code: KeyCode::new(104),
                id: "KC_H".into(),
                label: "h".into(),
                aliases: vec!["H".into()],
            },
        ];
        let keycodes_dto = keyforge_protocol::KeycodeRegistryDto {
            definitions: keycodes.into_iter().map(Into::into).collect(),
        };
        let keycodes_json = serde_json::to_string_pretty(&keycodes_dto)?;
        self.write_file("user/config/keycodes.json", &keycodes_json)
            .await?;
        self.write_file("system/config/keycodes.json", &keycodes_json)
            .await?;
        // Special requirement for FsProvider in integration tests: some builders expect keycodes at the root
        self.write_file("keycodes.json", &keycodes_json).await?;

        // 2. Cost Matrix
        let mut static_costs = HashMap::new();
        let mut base_costs = RowCosts::new();
        base_costs.insert(
            RowIndex::new(0),
            keyforge_model::types::Score::from_f32(100.0).map_err(|e| anyhow::anyhow!(e))?,
        );

        let fingers_def = FingerDefinition::Standard(keyforge_model::cost_model::FingerReach {
            base: base_costs,
            inner: HashMap::default(),
            outer: HashMap::default(),
        });

        let mut fingers = HashMap::new();
        fingers.insert(
            "thumb".into(),
            FingerDefinition::Thumb(HashMap::from([(
                "pos_1".into(),
                keyforge_model::types::Score::from_f32(100.0).map_err(|e| anyhow::anyhow!(e))?,
            )])),
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
        let cost_model_dto = keyforge_protocol::CostModelDto::from(cost_model);
        let cost_json = serde_json::to_string_pretty(&cost_model_dto)?;
        self.write_file("user/weights/cost.json", &cost_json)
            .await?;
        self.write_file("user/weights/default.json", &cost_json)
            .await?;
        // Some tests expect cost_matrix.json at the root
        self.write_file("cost_matrix.json", &cost_json).await?;

        // 3. Keyboard
        let geometry = KeyboardGeometry::new(
            vec![
                KeyNode::builder()
                    .index(keyforge_model::types::KeyIndex::new(0))
                    .x(SpatialUnit::from_f32(0.0))
                    .y(SpatialUnit::from_f32(0.0))
                    .hand(HandIndex::new(0))
                    .finger(FingerIndex::new(1))
                    .row(RowIndex::new(0))
                    .col(ColIndex::new(0))
                    .build(),
                KeyNode::builder()
                    .index(keyforge_model::types::KeyIndex::new(1))
                    .x(SpatialUnit::from_f32(1.0))
                    .y(SpatialUnit::from_f32(0.0))
                    .hand(HandIndex::new(0))
                    .finger(FingerIndex::new(2))
                    .row(RowIndex::new(0))
                    .col(ColIndex::new(1))
                    .build(),
            ],
            vec![
                keyforge_model::types::KeyIndex::new(0),
                keyforge_model::types::KeyIndex::new(1),
            ],
            vec![],
            vec![],
            keyforge_model::types::RowIndex::new(0),
        );

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
        let kb_json =
            serde_json::to_string_pretty(&keyforge_protocol::KeyboardDefinitionDto::from(kb_def))?;
        self.write_file("user/keyboards/test_kb.json", &kb_json)
            .await?;

        // 4. Legacy Directory-style Corpus (test_corpus)
        let corpus_json = r#"[{"s": "a", "f": 100}, {"s": "b", "f": 50}]"#;
        self.write_file("user/corpora/test_corpus/1grams.json", corpus_json)
            .await?;
        self.write_file("user/corpora/test_corpus/2grams.json", "[]")
            .await?;
        self.write_file("user/corpora/test_corpus/3grams.json", "[]")
            .await?;
        self.write_file("user/corpora/test_corpus/words.json", "[]")
            .await?;

        // 5. New Standard Serialized Corpus (en_small.json)
        let mut en_small = Corpus::default();
        let mut freqs = en_small.char_freqs.to_vec();
        freqs[116] = 1000; // 't'
        freqs[104] = 800; // 'h'
        en_small.char_freqs = Arc::from(freqs);
        en_small.bigrams = Arc::from(vec![(116, 104, 500)]); // 'th'
        let en_small_json =
            serde_json::to_string_pretty(&keyforge_protocol::CorpusDto::from(en_small))?;
        self.write_file("en_small.json", &en_small_json).await?;

        Ok(self)
    }

    /// Populates the workspace with "poison pill" assets designed to fail if constraints are ignored.
    ///
    /// # Errors
    /// Returns `anyhow::Result` if IO error occurs.
    pub async fn with_poison_pill(self) -> anyhow::Result<Self> {
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
        self.write_file("user/keyboards/poison_keyboard.json", kb_json)
            .await?;

        // Poison Weights: Massive penalty for High-freq char in Low-tier slot.
        let weights_json = r#"{
            "penalty_high_in_low": 1000000.0
        }"#;
        self.write_file("user/weights/poison_weights.json", weights_json)
            .await?;

        // Poison Corpus: 'e' is high freq.
        let corpus_json = r#"[{"s": "e", "f": 1000}]"#;
        self.write_file("user/corpora/poison_corpus/1grams.json", corpus_json)
            .await?;
        self.write_file("user/corpora/poison_corpus/2grams.json", "[]")
            .await?;
        self.write_file("user/corpora/poison_corpus/3grams.json", "[]")
            .await?;
        self.write_file("user/corpora/poison_corpus/words.json", "[]")
            .await?;

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
        self.write_file("user/weights/poison_cost.json", cost_json)
            .await?;

        Ok(self)
    }

    /// Writes a file to the workspace relative to the root asynchronously.
    ///
    /// # Errors
    /// Returns `anyhow::Result` if the directory cannot be created or the file cannot be written.
    pub async fn write_file(&self, path: &str, content: &str) -> anyhow::Result<()> {
        let target = self.root.join(path);
        if let Some(parent) = target.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(target, content).await?;
        Ok(())
    }

    // -- Path Helpers --

    #[must_use]
    pub fn keyboard_path(&self, name: &str) -> PathBuf {
        self.root
            .join("user/keyboards")
            .join(format!(/* "{name}.json" */ "{name}.json"))
    }

    #[must_use]
    pub fn cost_path(&self, name: &str) -> PathBuf {
        self.root.join("user/weights").join(name)
    }

    #[must_use]
    pub fn weights_path(&self, name: &str) -> PathBuf {
        self.root
            .join("user/weights")
            .join(format!(/* "{name}.json" */ "{name}.json"))
    }

    #[must_use]
    pub fn keycodes_path(&self, _name: &str) -> PathBuf {
        self.root.join("user/config/keycodes.json")
    }
}

// Re-export for convenience
pub use keyforge_model::constants;

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hermetic_workspace_lifecycle() {
        let ws = HermeticWorkspace::new()
            .await
            .expect("init failed")
            .with_default_assets()
            .await
            .expect("assets failed")
            .with_poison_pill()
            .await
            .expect("poison failed");

        // Check core files
        assert!(ws.root.exists());
        assert!(ws.keyboard_path("test_kb").exists());
        assert!(ws.weights_path("poison_weights").exists());
        assert!(ws.keycodes_path("default").exists());

        // Check corpus dir
        assert!(ws
            .root
            .join("user/corpora/test_corpus/1grams.json")
            .exists());

        // Check en_small
        assert!(ws.root.join("en_small.json").exists());
    }

    #[tokio::test]
    async fn test_hermetic_workspace_path_helpers() {
        let ws = HermeticWorkspace::new().await.unwrap();
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
