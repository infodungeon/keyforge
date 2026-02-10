// libs/keyforge-compute/tests/builder_integration.rs

#[keyforge_testing_macros::kf_test]
mod builder_tests {
    use super::*;
    use keyforge_adapter::loader::{AssetLoader, LoaderResult};
    use keyforge_compute::SessionBuilder;
    use keyforge_model::geometry::{KeyboardDefinition, KeyboardGeometry, KeyboardMeta};
    use keyforge_model::keycodes::KeycodeRegistry;
    use keyforge_model::{Asset, Corpus, CostModel};
    use std::any::Any;
    use std::collections::HashMap;
    use std::path::Path;
    use std::sync::Arc;

    #[derive(Debug)]
    struct MockLoader {
        assets: Arc<dyn Any + Send + Sync>,
        root: keyforge_boundary::SafePath,
    }

    #[async_trait::async_trait]
    impl AssetLoader for MockLoader {
        async fn load<T: Asset>(&self, id: &str) -> LoaderResult<Arc<T>> {
            let tid = std::any::TypeId::of::<T>();

            // Special handling for keycodes which are often requested by string literal
            if tid == std::any::TypeId::of::<keyforge_protocol::KeycodeRegistryDto>() {
                let reg = keyforge_protocol::KeycodeRegistryDto::default();
                let any_kc = Arc::new(reg) as Arc<dyn Any + Send + Sync>;
                return Ok(any_kc.downcast::<T>().expect("Downcast failed"));
            }

            // Special handling for cost model (DTO)
            if tid == std::any::TypeId::of::<keyforge_protocol::CostModelDto>() {
                let mut model = keyforge_protocol::CostModelDto::default();
                let mut hand_def = keyforge_protocol::HandDefinitionDto::default();

                for finger in &["thumb", "index", "middle", "ring", "pinky"] {
                    hand_def.fingers.insert(
                        (*finger).to_string(),
                        keyforge_protocol::FingerDefinitionDto::Standard(
                            keyforge_protocol::FingerReachDto::default(),
                        ),
                    );
                }

                let mut model_def = keyforge_protocol::ModelDefinitionDto::default();
                model_def
                    .static_costs
                    .insert("universal_hand".to_string(), hand_def);

                model.models.insert("default".to_string(), model_def);
                let any_model = Arc::new(model) as Arc<dyn Any + Send + Sync>;
                return Ok(any_model.downcast::<T>().expect("Downcast failed"));
            }

            // Fallback to the generic injected asset (KeyboardDefinition in this test)
            if let Ok(arc) = self.assets.clone().downcast::<T>() {
                return Ok(arc);
            }

            Err(keyforge_model::error::ForgeError::NotFound(format!(
                "Mock asset not found: {id}"
            )))
        }

        async fn load_corpus(
            &self,
            _sources: &[keyforge_model::config::CorpusSource],
        ) -> LoaderResult<Arc<Corpus>> {
            Ok(Arc::new(Corpus::default()))
        }

        fn root(&self) -> &keyforge_boundary::SafePath {
            &self.root
        }

        async fn get_hash(
            &self,
            _category: keyforge_model::asset::AssetCategory,
            _id: &str,
        ) -> LoaderResult<String> {
            Ok("mock_hash".to_string())
        }
    }

    #[tokio::test]
    async fn test_builder_full_chain() {
        let kb_def = KeyboardDefinition {
            meta: KeyboardMeta::default(),
            geometry: KeyboardGeometry::new(
                vec![keyforge_model::KeyNode::default()],
                vec![keyforge_model::types::KeyIndex::new(0)],
                vec![],
                vec![],
                keyforge_model::types::RowIndex::new(0),
            ),
            layouts: HashMap::default(),
        };

        let loader = MockLoader {
            assets: Arc::new(kb_def.clone()),
            root: keyforge_boundary::SafePath::from_trusted_root_path(std::path::PathBuf::from(
                ".",
            )),
        };

        let builder = SessionBuilder::new(&loader)
            .with_keyboard_def(Arc::new(kb_def))
            .with_corpus(&[])
            .await
            .unwrap()
            .with_cost_matrix(&keyforge_model::config::CostMatrixSource::default())
            .await
            .unwrap()
            .with_keycodes("default")
            .await
            .unwrap();

        let session = builder.build().unwrap();
        assert_eq!(session.engine.key_count(), 1);
    }
}
