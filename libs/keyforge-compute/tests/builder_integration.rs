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
    }

    #[async_trait::async_trait]
    impl AssetLoader for MockLoader {
        async fn load<T: Asset>(&self, id: &str) -> LoaderResult<Arc<T>> {
            let tid = std::any::TypeId::of::<T>();

            // Special handling for keycodes which are often requested by string literal
            if tid == std::any::TypeId::of::<KeycodeRegistry>() {
                let reg = KeycodeRegistry::new_with_defaults();
                let any_kc = Arc::new(reg) as Arc<dyn Any + Send + Sync>;
                return Ok(any_kc.downcast::<T>().expect("Downcast failed"));
            }

            // Special handling for cost model
            if tid == std::any::TypeId::of::<CostModel>() {
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

        fn root(&self) -> &Path {
            Path::new(".")
        }
    }

    #[tokio::test]
    async fn test_builder_full_chain() {
        let kb_def = KeyboardDefinition {
            meta: KeyboardMeta::default(),
            geometry: KeyboardGeometry {
                keys: vec![keyforge_model::KeyNode::default()],
                prime_slots: vec![keyforge_model::types::KeyIndex(0)],
                med_slots: vec![],
                low_slots: vec![],
                home_row: keyforge_model::types::RowIndex(0),
            },
            layouts: HashMap::default(),
        };

        let loader = MockLoader {
            assets: Arc::new(kb_def.clone()),
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
