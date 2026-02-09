// libs/keyforge-compute/tests/runtime_integration.rs

#[keyforge_testing_macros::kf_test]
mod runtime_tests {
    use super::*;
    use async_trait::async_trait;
    use keyforge_adapter::loader::{AssetLoader, LoaderResult};
    use keyforge_compute::Runtime;
    use keyforge_model::geometry::{KeyboardDefinition, KeyboardMeta};
    use keyforge_model::{Asset, Corpus};
    use std::path::Path;
    use std::sync::Arc;

    #[derive(Debug)]
    struct MockLoader {
        root: keyforge_model::types::path::SafePath,
    }

    impl MockLoader {
        fn new() -> Self {
            Self {
                root: keyforge_model::types::path::SafePath::from_trusted_root_path(
                    std::path::PathBuf::from("."),
                ),
            }
        }
    }

    #[async_trait]
    impl AssetLoader for MockLoader {
        async fn load<T: Asset>(&self, _id: &str) -> LoaderResult<Arc<T>> {
            let tid = std::any::TypeId::of::<T>();

            if tid == std::any::TypeId::of::<keyforge_model::keycodes::KeycodeRegistry>() {
                let reg = keyforge_model::keycodes::KeycodeRegistry::new_with_defaults();
                let any_kc = Arc::new(reg) as Arc<dyn std::any::Any + Send + Sync>;
                return Ok(any_kc.downcast::<T>().expect("Downcast failed"));
            }

            if tid == std::any::TypeId::of::<keyforge_model::CostModel>() {
                let mut model = keyforge_model::cost_model::CostModel::default();
                let mut hand_def = keyforge_model::cost_model::HandDefinition::default();
                for finger in &["thumb", "index", "middle", "ring", "pinky"] {
                    hand_def.fingers.insert(
                        (*finger).to_string(),
                        keyforge_model::cost_model::FingerDefinition::Standard(
                            keyforge_model::cost_model::FingerReach::default(),
                        ),
                    );
                }
                let mut model_def = keyforge_model::cost_model::ModelDefinition::default();
                model_def
                    .static_costs
                    .insert("universal_hand".to_string(), hand_def);
                model.models.insert("default".to_string(), model_def);

                let any_model = Arc::new(model) as Arc<dyn std::any::Any + Send + Sync>;
                return Ok(any_model.downcast::<T>().expect("Downcast failed"));
            }

            let kb_any = Arc::new(KeyboardDefinition {
                meta: KeyboardMeta::default(),
                ..Default::default()
            }) as Arc<dyn std::any::Any + Send + Sync>;
            Ok(kb_any.downcast::<T>().expect("Downcast failed"))
        }

        async fn load_corpus(
            &self,
            _sources: &[keyforge_model::config::CorpusSource],
        ) -> LoaderResult<Arc<Corpus>> {
            Ok(Arc::new(Corpus::default()))
        }

        fn root(&self) -> &keyforge_model::types::path::SafePath {
            &self.root
        }

        async fn get_hash(
            &self,
            _category: keyforge_model::asset::AssetCategory,
            _id: &str,
        ) -> LoaderResult<String> {
            Ok("mock".to_string())
        }
    }

    #[tokio::test]
    async fn test_runtime_execution() {
        let loader = MockLoader::new();
        let mut kb_def = KeyboardDefinition::default();
        kb_def
            .geometry
            .keys
            .push(keyforge_model::KeyNode::default());
        kb_def
            .geometry
            .prime_slots
            .push(keyforge_model::types::KeyIndex::new(0));

        let session = keyforge_compute::SessionBuilder::new(&loader)
            .with_keyboard_def(Arc::new(kb_def))
            .with_corpus(&[])
            .await
            .unwrap()
            .with_cost_matrix(&keyforge_model::config::CostMatrixSource::default())
            .await
            .unwrap()
            .with_keycodes("default")
            .await
            .unwrap()
            .build()
            .unwrap();

        let runtime = Runtime::from(session);
        let logger = keyforge_evolution::NoOpCallback;
        // Fix: Pass logger by value to satisfy ProgressCallback + 'static bound
        let res = runtime.run_optimization(logger, &[]).await;
        assert!(res.is_ok());
    }
}
