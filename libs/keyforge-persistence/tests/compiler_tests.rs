#[keyforge_testing_macros::kf_test]
mod integration_tests {
    use super::*;
    use keyforge_compute::loader::{AssetLoader, LoaderResult};
    use keyforge_model::cost_model::CostModel;
    use keyforge_model::geometry::{KeyboardDefinition, KeyboardGeometry};
    use keyforge_model::keycodes::KeycodeRegistry;
    use keyforge_model::{config::Config, config::CorpusSource, Asset, Corpus};
    use keyforge_persistence::compiler::compile_request;
    use std::any::Any;
    use std::sync::Arc;

    #[derive(Debug)]
    struct MockLoader;

    #[async_trait::async_trait]
    impl AssetLoader for MockLoader {
        async fn load<T: Asset>(&self, _id: &str) -> LoaderResult<Arc<T>> {
            let any_kb = Arc::new(KeyboardDefinition {
                meta: Default::default(),
                geometry: KeyboardGeometry {
                    keys: vec![Default::default()],
                    prime_slots: vec![],
                    med_slots: vec![],
                    low_slots: vec![],
                    home_row: 0,
                },
                layouts: Default::default(),
            }) as Arc<dyn Any + Send + Sync>;

            if let Ok(arc) = any_kb.downcast::<T>() {
                return Ok(arc);
            }

            let json = r#"{
            "meta": { "version": "2.0", "description": "Test", "unit": "pts" },
            "models": {
                "model_a_row_staggered": {
                    "description": "test",
                    "static_costs": {}
                }
            },
            "dynamic_rules": { "sequence_modifiers": {}, "penalties": {}, "constraints": {} }
        }"#;
            let model: CostModel = serde_json::from_str(json).unwrap();
            let any_model = Arc::new(model) as Arc<dyn Any + Send + Sync>;
            if let Ok(arc) = any_model.downcast::<T>() {
                return Ok(arc);
            }

            let any_kc = Arc::new(KeycodeRegistry::default()) as Arc<dyn Any + Send + Sync>;
            if let Ok(arc) = any_kc.downcast::<T>() {
                return Ok(arc);
            }

            Err(keyforge_model::error::ForgeError::NotFound(_id.to_string()))
        }

        async fn load_corpus(&self, _sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>> {
            Ok(Arc::new(Corpus::default()))
        }
    }

    #[tokio::test]
    async fn test_compile_request_success() {
        let loader = MockLoader;
        let mut config = Config::default();
        config.keyboard = "test_kb".into();
        config.corpora = vec![CorpusSource {
            id: "en".into(),
            weight: 1.0,
            hash: None,
        }];

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
                let mut kb = KeyboardDefinition::default();
                kb.geometry.keys.push(keyforge_model::KeyNode {
                    label: "A".into(),
                    ..Default::default()
                });
                kb.geometry.prime_slots.push(keyforge_model::KeyIndex(0));
                kb.layouts.insert("qwerty".into(), "A".into());

                let any_kb = Arc::new(kb) as Arc<dyn Any + Send + Sync>;
                if let Ok(arc) = any_kb.downcast::<T>() {
                    return Ok(arc);
                }

                let json = r#"{
                "meta": { "version": "2.0", "description": "T", "unit": "pts" },
                "models": { "model_a_row_staggered": { "description": "t", "static_costs": {} } },
                "dynamic_rules": { "sequence_modifiers": {}, "penalties": {}, "constraints": {} }
            }"#;
                let model: CostModel = serde_json::from_str(json).unwrap();
                let any_model = Arc::new(model) as Arc<dyn Any + Send + Sync>;
                if let Ok(arc) = any_model.downcast::<T>() {
                    return Ok(arc);
                }

                let mut reg = KeycodeRegistry::new_with_defaults();
                reg.definitions
                    .push(keyforge_model::keycodes::KeycodeDefinition {
                        code: keyforge_model::KeyCode(10),
                        id: "A".into(),
                        label: "a".into(),
                        aliases: vec![],
                    });
                reg.rebuild_maps();
                let any_kc = Arc::new(reg) as Arc<dyn Any + Send + Sync>;
                if let Ok(arc) = any_kc.downcast::<T>() {
                    return Ok(arc);
                }

                Err(keyforge_model::error::ForgeError::NotFound(_id.to_string()))
            }
            async fn load_corpus(&self, _sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>> {
                Ok(Arc::new(Corpus::default()))
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
        }

        let loader = FailingLoader;
        let mut config = Config::default();
        config.keyboard = "kb".into();

        // Fail keyboard
        let res = compile_request(&loader, &config).await;
        assert!(res.is_err());
    }
}
