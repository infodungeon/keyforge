// libs/keyforge-persistence/tests/compiler_tests.rs

#[keyforge_testing_macros::kf_test]
mod integration_tests {
    use super::*;
    use keyforge_adapter::loader::{AssetLoader, LoaderResult};
    use keyforge_model::cost_model::CostModel;
    use keyforge_model::geometry::{KeyNode, KeyboardDefinition, KeyboardGeometry, KeyboardMeta};
    use keyforge_model::keycodes::KeycodeRegistry;
    use keyforge_model::types::{KeyIndex, KeyCode};
    use keyforge_model::{config::Config, config::CorpusSource, Asset, Corpus};
    use keyforge_persistence::compiler::compile_request;
    use std::any::Any;
    use std::collections::HashMap;
    use std::path::Path;
    use std::sync::Arc;

    #[derive(Debug)]
    struct MockLoader {
        assets: Arc<dyn Any + Send + Sync>,
    }

    #[async_trait::async_trait]
    impl AssetLoader for MockLoader {
        async fn load<T: Asset>(&self, _id: &str) -> LoaderResult<Arc<T>> {
            if std::any::TypeId::of::<T>() == std::any::TypeId::of::<KeyboardDefinition>() {
                if let Ok(arc) = self.assets.clone().downcast::<T>() {
                    return Ok(arc);
                }
            }

            if std::any::TypeId::of::<T>() == std::any::TypeId::of::<CostModel>() {
                let json = r#"{
                    "meta": { "version": "2.0", "description": "T", "unit": "pts" },
                    "models": { 
                        "model_a_row_staggered": { 
                            "description": "t", 
                            "static_costs": {
                                "universal_hand": {
                                    "thumb": {"base": {"r0": 1.0}},
                                    "index": {"base": {"r0": 1.0}},
                                    "middle": {"base": {"r0": 1.0}},
                                    "ring": {"base": {"r0": 1.0}},
                                    "pinky": {"base": {"r0": 1.0}}
                                }
                            } 
                        } 
                    },
                    "dynamic_rules": { "sequence_modifiers": {}, "penalties": {}, "constraints": {} }
                }"#;
                let model: CostModel = serde_json::from_str(json).map_err(|e| keyforge_model::error::ForgeError::Serde(e.to_string()))?;
                let any_model = Arc::new(model) as Arc<dyn Any + Send + Sync>;
                return Ok(any_model.downcast::<T>().expect("Downcast failed"));
            }

            if std::any::TypeId::of::<T>() == std::any::TypeId::of::<KeycodeRegistry>() {
                let reg = KeycodeRegistry::new_with_defaults();
                let any_kc = Arc::new(reg) as Arc<dyn Any + Send + Sync>;
                return Ok(any_kc.downcast::<T>().expect("Downcast failed"));
            }

            Err(keyforge_model::error::ForgeError::NotFound("Mock".into()))
        }

        async fn load_corpus(&self, _sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>> {
            Ok(Arc::new(Corpus::default()))
        }

        fn root(&self) -> &Path {
            Path::new(".")
        }
    }

    #[tokio::test]
    async fn test_compile_request_basic() {
        let kb_def = KeyboardDefinition {
            meta: KeyboardMeta::default(),
            geometry: KeyboardGeometry {
                keys: vec![KeyNode::default()],
                prime_slots: vec![KeyIndex::new(0)],
                med_slots: vec![],
                low_slots: vec![],
                home_row: keyforge_model::types::RowIndex(0),
            },
            layouts: HashMap::default(),
        };

        let loader = MockLoader {
            assets: Arc::new(kb_def),
        };

        let config = Config::default();
        let res = compile_request(&loader, &config).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_compile_request_qwerty() {
        #[derive(Debug)]
        struct QwertyLoader;
        #[async_trait::async_trait]
        impl AssetLoader for QwertyLoader {
            async fn load<T: Asset>(&self, _id: &str) -> LoaderResult<Arc<T>> {
                if std::any::TypeId::of::<T>() == std::any::TypeId::of::<KeyboardDefinition>() {
                    let mut kb = KeyboardDefinition::default();
                    kb.geometry.keys.push(keyforge_model::KeyNode {
                        label: "A".into(),
                        ..Default::default()
                    });
                    kb.geometry.prime_slots.push(keyforge_model::KeyIndex::new(0));
                    kb.layouts.insert("qwerty".into(), "A".into());

                    let any_kb = Arc::new(kb) as Arc<dyn Any + Send + Sync>;
                    return Ok(any_kb.downcast::<T>().expect("Downcast failed"));
                }

                if std::any::TypeId::of::<T>() == std::any::TypeId::of::<CostModel>() {
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
                            } 
                        },
                        "dynamic_rules": { "sequence_modifiers": {}, "penalties": {}, "constraints": {} }
                    }"#;
                    let model: CostModel = serde_json::from_str(json).map_err(|e| keyforge_model::error::ForgeError::Serde(e.to_string()))?;
                    let any_model = Arc::new(model) as Arc<dyn Any + Send + Sync>;
                    return Ok(any_model.downcast::<T>().expect("Downcast failed"));
                }

                if std::any::TypeId::of::<T>() == std::any::TypeId::of::<KeycodeRegistry>() {
                    let mut reg = KeycodeRegistry::new_with_defaults();
                    reg.definitions
                        .push(keyforge_model::keycodes::KeycodeDefinition {
                            code: KeyCode::new(10),
                            id: "A".into(),
                            label: "a".into(),
                            aliases: vec![],
                        });
                    reg.rebuild_maps();
                    let any_kc = Arc::new(reg) as Arc<dyn Any + Send + Sync>;
                    return Ok(any_kc.downcast::<T>().expect("Downcast failed"));
                }

                Err(keyforge_model::error::ForgeError::NotFound(_id.to_string()))
            }
            async fn load_corpus(&self, _sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>> {
                Ok(Arc::new(Corpus::default()))
            }
            fn root(&self) -> &Path {
                Path::new(".")
            }
        }

        let loader = QwertyLoader;
        let mut config = Config::default();
        config.keyboard = "kb".into();

        let res = compile_request(&loader, &config).await.unwrap();
        assert!(res.initial_layout.is_some());
    }

    #[tokio::test]
    async fn test_compile_request_failures() {
        #[derive(Debug)]
        struct FailingLoader;
        #[async_trait::async_trait]
        impl AssetLoader for FailingLoader {
            async fn load<T: Asset>(&self, _id: &str) -> LoaderResult<Arc<T>> {
                Err(keyforge_model::error::ForgeError::NotFound(_id.to_string()))
            }
            async fn load_corpus(&self, _sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>> {
                Err(keyforge_model::error::ForgeError::NotFound(
                    "corpus".to_string(),
                ))
            }
            fn root(&self) -> &Path {
                Path::new(".")
            }
        }

        let loader = FailingLoader;
        let mut config = Config::default();
        config.keyboard = "kb".into();

        // Fail keyboard
        let res = compile_request(&loader, &config).await;
        assert!(res.is_err());
    }
}