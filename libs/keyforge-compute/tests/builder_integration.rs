#[keyforge_testing_macros::kf_test]
#[allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::items_after_statements,
    clippy::too_many_lines,
    clippy::float_cmp,
    clippy::similar_names,
    clippy::default_trait_access,
    clippy::print_stdout
)]
mod integration_tests {
    use super::*;
    use keyforge_model::geometry::KeyboardMeta;
    use std::collections::HashMap;

    use keyforge_compute::{
        OptimizationControl, ProgressCallback, Runtime, ScoringSession, SessionBuilder,
    };
    use keyforge_model::config::{CorpusSource, CostMatrixSource};
    use keyforge_model::geometry::KeyboardDefinition;
    use keyforge_model::keycodes::KeycodeRegistry;
    use keyforge_model::loader::{AssetLoader, LoaderResult};
    use keyforge_model::types::RowIndex;
    use keyforge_model::{
        Asset, Corpus, CostModel, KeyNode, Keyboard, Layout, Rubric, SearchConfig,
    };
    use keyforge_physics::EngineFactory;
    use keyforge_protocol::BiometricSample;
    use std::any::Any;
    use std::sync::Arc;

    #[derive(Debug)]
    struct MockLoader;
    #[async_trait::async_trait]
    impl AssetLoader for MockLoader {
        async fn load<T: Asset>(&self, id: &str) -> LoaderResult<Arc<T>> {
            if id == "missing" {
                return Err(keyforge_model::error::ForgeError::NotFound(id.into()));
            }

            let kb_any = Arc::new(KeyboardDefinition {
                geometry: keyforge_model::geometry::KeyboardGeometry {
                    keys: vec![keyforge_model::KeyNode::default()],
                    prime_slots: vec![keyforge_model::types::KeyIndex(0)],
                    ..Default::default()
                },
                ..Default::default()
            }) as Arc<dyn Any + Send + Sync>;
            if let Ok(arc) = kb_any.downcast::<T>() {
                return Ok(arc);
            }

            let json = r#"{
            "meta": { "version": "2.0", "description": "T", "unit": "pts" },
            "models": { 
                "model_a_row_staggered": { 
                    "description": "t", 
                    "static_costs": {
                        "universal_hand": {
                            "thumb": {"base": {"0": 1.0}},
                            "index": {"base": {"0": 1.0}},
                            "middle": {"base": {"0": 1.0}},
                            "ring": {"base": {"0": 1.0}},
                            "pinky": {"base": {"0": 1.0}}
                        }
                    } 
                },
                "model_a_ansi": { 
                    "description": "t", 
                    "static_costs": {
                        "universal_hand": {
                            "thumb": {"base": {"0": 1.0}},
                            "index": {"base": {"0": 1.0}},
                            "middle": {"base": {"0": 1.0}},
                            "ring": {"base": {"0": 1.0}},
                            "pinky": {"base": {"0": 1.0}}
                        }
                    } 
                },
                "model_ortho": { 
                    "description": "t", 
                    "static_costs": {
                        "universal_hand": {
                            "thumb": {"base": {"0": 1.0}},
                            "index": {"base": {"0": 1.0}},
                            "middle": {"base": {"0": 1.0}},
                            "ring": {"base": {"0": 1.0}},
                            "pinky": {"base": {"0": 1.0}}
                        }
                    } 
                }
            },
            "dynamic_rules": { "sequence_modifiers": {}, "penalties": {}, "constraints": {} }
        }"#;
            let model: CostModel = serde_json::from_str(json).unwrap();
            let any_model = Arc::new(model) as Arc<dyn Any + Send + Sync>;
            if let Ok(arc) = any_model.downcast::<T>() {
                return Ok(arc);
            }

            let kc_any = Arc::new(KeycodeRegistry::default()) as Arc<dyn Any + Send + Sync>;
            if let Ok(arc) = kc_any.downcast::<T>() {
                return Ok(arc);
            }

            Err(keyforge_model::error::ForgeError::NotFound(id.to_string()))
        }
        async fn load_corpus(&self, _sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>> {
            Ok(Arc::new(Corpus::default()))
        }
    }

    fn test_search_config() -> SearchConfig {
        SearchConfig::Annealing {
            steps: 100,
            start_temp: 10.0,
            end_temp: 0.1,
            seed: 42,
            patience: 10,
            reheats: 0,
            reheat_factor: 0.5,
            include_thumbs: false,
        }
    }

    fn setup_runtime() -> Runtime {
        let kb = Keyboard::new(vec![KeyNode::default()], 0, "test".into()).unwrap();
        let mut cm = CostModel::default();
        let mut fingers = std::collections::HashMap::new();
        for finger in ["thumb", "index", "middle", "ring", "pinky"] {
            fingers.insert(
                finger.to_string(),
                keyforge_model::cost_model::FingerDefinition::Standard(
                    keyforge_model::cost_model::FingerReach {
                        base: std::collections::HashMap::from([(RowIndex(0), 1.0)]),
                        ..Default::default()
                    },
                ),
            );
        }
        cm.models.insert(
            "model_a_row_staggered".into(),
            keyforge_model::cost_model::ModelDefinition {
                description: "test".into(),
                static_costs: std::collections::HashMap::from([(
                    "universal_hand".to_string(),
                    keyforge_model::cost_model::HandDefinition { fingers },
                )]),
            },
        );
        let engine = EngineFactory::new_exact(keyforge_physics::EngineCompilationContext {
            keyboard: &kb,
            corpus: &Corpus::default(),
            rubric: &Rubric::default(),
            cost_model: &cm,
        })
        .unwrap();
        let registry = Arc::new(KeycodeRegistry::new_with_defaults());
        Runtime::new(Arc::from(engine), registry, test_search_config())
    }

    #[test]
    fn test_runtime_methods() {
        struct NoOpCallback;
        impl ProgressCallback for NoOpCallback {
            fn on_progress(
                &self,
                _epoch: usize,
                _score: f32,
                _layout: &[keyforge_model::KeyCode],
                _ips: f32,
            ) -> OptimizationControl {
                OptimizationControl::Continue
            }
        }

        let mut rt = setup_runtime();
        let layout = Layout::new_unchecked(vec![keyforge_model::KeyCode(0)]);

        assert!(rt.score(&layout).is_ok());
        assert!(rt.analyze(&layout).is_ok());
        assert!(rt.suggest_improvements(&layout).is_ok());

        // Trigger include_thumbs branch
        rt.search_config = SearchConfig::Annealing {
            steps: 10,
            start_temp: 1.0,
            end_temp: 0.1,
            seed: 0,
            patience: 10,
            reheats: 0,
            reheat_factor: 0.5,
            include_thumbs: true,
        };
        assert!(rt.suggest_improvements(&layout).is_ok());

        assert!(rt.optimize(NoOpCallback, None, None).is_ok());
    }

    #[test]
    fn test_runtime_from_session() {
        let kb = Keyboard::new(vec![KeyNode::default()], 0, "test".into()).unwrap();
        let mut cm = CostModel::default();
        let mut fingers = std::collections::HashMap::new();
        for finger in ["thumb", "index", "middle", "ring", "pinky"] {
            fingers.insert(
                finger.to_string(),
                keyforge_model::cost_model::FingerDefinition::Standard(
                    keyforge_model::cost_model::FingerReach {
                        base: std::collections::HashMap::from([(RowIndex(0), 1.0)]),
                        ..Default::default()
                    },
                ),
            );
        }
        cm.models.insert(
            "model_a_row_staggered".into(),
            keyforge_model::cost_model::ModelDefinition {
                description: "test".into(),
                static_costs: std::collections::HashMap::from([(
                    "universal_hand".to_string(),
                    keyforge_model::cost_model::HandDefinition { fingers },
                )]),
            },
        );
        let engine = EngineFactory::new_exact(keyforge_physics::EngineCompilationContext {
            keyboard: &kb,
            corpus: &Corpus::default(),
            rubric: &Rubric::default(),
            cost_model: &cm,
        })
        .unwrap();
        let session = ScoringSession::new(
            Arc::from(engine),
            Arc::new(KeycodeRegistry::default()),
            test_search_config(),
        );

        let rt = Runtime::from(session);
        assert_eq!(rt.registry.definitions.len(), 0);
    }

    #[tokio::test]
    async fn test_session_builder_lifecycle() {
        let loader = MockLoader;
        let kb_def = Arc::new(KeyboardDefinition {
            meta: keyforge_model::geometry::KeyboardMeta {
                kb_type: "ortho".into(),
                ..Default::default()
            },
            geometry: keyforge_model::geometry::KeyboardGeometry {
                keys: vec![keyforge_model::KeyNode::default()],
                prime_slots: vec![keyforge_model::types::KeyIndex(0)],
                ..Default::default()
            },
            ..Default::default()
        });
        let corp = Arc::new(Corpus::default());
        let mut cm = CostModel::default();
        let mut fingers = std::collections::HashMap::new();
        for finger in ["thumb", "index", "middle", "ring", "pinky"] {
            fingers.insert(
                finger.to_string(),
                keyforge_model::cost_model::FingerDefinition::Standard(
                    keyforge_model::cost_model::FingerReach {
                        base: std::collections::HashMap::from([(RowIndex(0), 1.0)]),
                        ..Default::default()
                    },
                ),
            );
        }
        cm.models.insert(
            "model_ortho".into(),
            keyforge_model::cost_model::ModelDefinition {
                description: "test".into(),
                static_costs: std::collections::HashMap::from([(
                    "universal_hand".to_string(),
                    keyforge_model::cost_model::HandDefinition {
                        fingers: fingers.clone(),
                    },
                )]),
            },
        );
        cm.models.insert(
            "model_a_row_staggered".into(),
            keyforge_model::cost_model::ModelDefinition {
                description: "test".into(),
                static_costs: std::collections::HashMap::from([(
                    "universal_hand".to_string(),
                    keyforge_model::cost_model::HandDefinition {
                        fingers: fingers.clone(),
                    },
                )]),
            },
        );
        cm.models.insert(
            "model_a_ansi".into(),
            keyforge_model::cost_model::ModelDefinition {
                description: "test".into(),
                static_costs: std::collections::HashMap::from([(
                    "universal_hand".to_string(),
                    keyforge_model::cost_model::HandDefinition {
                        fingers: fingers.clone(),
                    },
                )]),
            },
        );
        let cm_arc = Arc::new(cm);

        let builder = SessionBuilder::new(&loader)
            .with_keyboard("kb")
            .await
            .unwrap()
            .with_keyboard_def(kb_def)
            .with_corpus(&[])
            .await
            .unwrap()
            .with_corpus_obj(corp)
            .with_cost_matrix(&CostMatrixSource::Predefined("cm".into()))
            .await
            .unwrap()
            .with_cost_model_obj(cm_arc)
            .with_keycodes("kc")
            .await
            .unwrap()
            .with_rubric(Rubric::default())
            .with_config(test_search_config())
            .with_biometrics(vec![BiometricSample {
                bigram: "th".into(),
                ms: 100.0,
                timestamp: 0,
            }]);

        let debug_str = format!("{builder:?}");
        assert!(debug_str.contains("SessionBuilder"));
        assert!(debug_str.contains("biometrics_count: 1"));

        let session = builder.build().unwrap();
        assert_eq!(session.registry.definitions.len(), 0);
    }

    #[tokio::test]
    async fn test_session_builder_missing_assets() {
        let loader = MockLoader;

        // 1. Missing keyboard
        let b1 = SessionBuilder::new(&loader);
        assert!(b1.build().is_err());

        // 2. Missing corpus
        let b2 = SessionBuilder::new(&loader)
            .with_keyboard("kb")
            .await
            .unwrap();
        assert!(b2.build().is_err());

        // 3. Missing cost model
        let b3 = SessionBuilder::new(&loader)
            .with_keyboard("kb")
            .await
            .unwrap()
            .with_corpus(&[])
            .await
            .unwrap();
        assert!(b3.build().is_err());

        // 4. Default registry and rubric
        let mut cm = CostModel::default();
        let mut fingers = std::collections::HashMap::new();
        for finger in ["thumb", "index", "middle", "ring", "pinky"] {
            fingers.insert(
                finger.to_string(),
                keyforge_model::cost_model::FingerDefinition::Standard(
                    keyforge_model::cost_model::FingerReach {
                        base: std::collections::HashMap::from([(RowIndex(0), 1.0)]),
                        ..Default::default()
                    },
                ),
            );
        }
        cm.models.insert(
            "model_ortho".into(),
            keyforge_model::cost_model::ModelDefinition {
                description: "test".into(),
                static_costs: std::collections::HashMap::from([(
                    "universal_hand".to_string(),
                    keyforge_model::cost_model::HandDefinition {
                        fingers: fingers.clone(),
                    },
                )]),
            },
        );
        cm.models.insert(
            "model_a_row_staggered".into(),
            keyforge_model::cost_model::ModelDefinition {
                description: "test".into(),
                static_costs: std::collections::HashMap::from([(
                    "universal_hand".to_string(),
                    keyforge_model::cost_model::HandDefinition {
                        fingers: fingers.clone(),
                    },
                )]),
            },
        );
        cm.models.insert(
            "model_a_ansi".into(),
            keyforge_model::cost_model::ModelDefinition {
                description: "test".into(),
                static_costs: std::collections::HashMap::from([(
                    "universal_hand".to_string(),
                    keyforge_model::cost_model::HandDefinition {
                        fingers: fingers.clone(),
                    },
                )]),
            },
        );
        let b4 = SessionBuilder::new(&loader)
            .with_keyboard_def(Arc::new(KeyboardDefinition {
                geometry: keyforge_model::geometry::KeyboardGeometry {
                    keys: vec![keyforge_model::KeyNode::default()],
                    prime_slots: vec![keyforge_model::types::KeyIndex(0)],
                    ..Default::default()
                },
                ..Default::default()
            }))
            .with_corpus_obj(Arc::new(Corpus::default()))
            .with_cost_model_obj(Arc::new(cm))
            .with_config(test_search_config());
        let session = b4.build().unwrap();
        assert_eq!(session.registry.definitions.len(), 0); // Uses default (empty) registry
    }

    #[tokio::test]
    async fn test_session_builder_invalid_keyboard() {
        let loader = MockLoader;
        let mut kb = KeyboardDefinition::default();
        // Keyboard with keys but no slots coverage -> validation fails in Keyboard::new
        kb.geometry.keys.push(keyforge_model::KeyNode::default());

        let builder = SessionBuilder::new(&loader)
            .with_keyboard_def(Arc::new(kb))
            .with_corpus_obj(Arc::new(Corpus::default()))
            .with_cost_model_obj(Arc::new(CostModel::default()));

        assert!(builder.build().is_err());
    }

    #[tokio::test]
    async fn test_session_builder_physics_error() {
        let loader = MockLoader;
        let kb_def = Arc::new(KeyboardDefinition {
            geometry: keyforge_model::geometry::KeyboardGeometry {
                keys: vec![keyforge_model::KeyNode::default()],
                prime_slots: vec![keyforge_model::types::KeyIndex(0)],
                ..Default::default()
            },
            ..Default::default()
        });

        let builder = SessionBuilder::new(&loader).with_keyboard_def(kb_def);
        let res = builder.build();
        assert!(matches!(
            res,
            Err(keyforge_model::error::ForgeError::Config(_))
        ));
    }
}
