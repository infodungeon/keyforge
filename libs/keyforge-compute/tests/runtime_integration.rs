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

    // libs/keyforge-compute/tests/runtime_integration.rs

    use async_trait::async_trait;
    use keyforge_adapter::loader::{AssetLoader, LoaderResult};
    use keyforge_compute::{Runtime, SessionBuilder};
    use keyforge_model::{types::KeyCode, Asset, Corpus, KeyNode, KeyboardDefinition, Layout};
    use std::any::Any;
    use std::sync::Arc;

    #[derive(Debug)]
    struct MockLoader;

    #[async_trait]
    impl AssetLoader for MockLoader {
        async fn load<T: Asset>(&self, _id: &str) -> LoaderResult<Arc<T>> {
            let kb_any = Arc::new(KeyboardDefinition {
                meta: KeyboardMeta::default(),
                geometry: keyforge_model::geometry::KeyboardGeometry {
                    keys: vec![KeyNode {
                        index: 0,
                        x: 0.0,
                        y: 0.0,
                        ..Default::default()
                    }],
                    prime_slots: vec![keyforge_model::KeyIndex(0)],
                    med_slots: vec![],
                    low_slots: vec![],
                    home_row: 0,
                },
                layouts: HashMap::default(),
            }) as Arc<dyn Any + Send + Sync>;

            if let Ok(arc) = kb_any.downcast::<T>() {
                return Ok(arc);
            }

            let json = r#"{
            "meta": { "version": "2.0", "description": "Test", "unit": "pts" },
            "models": {
                "model_a_row_staggered": {
                    "description": "test",
                    "static_costs": {"universal_hand": {"thumb": {"base": {"r0": 1.0}}, "index": {"base": {"r0": 1.0}}}}
                }
            },
            "dynamic_rules": { "sequence_modifiers": {}, "penalties": {}, "constraints": {} }
        }"#;
            let model: keyforge_model::cost_model::CostModel = serde_json::from_str(json).unwrap();
            let any_model = Arc::new(model) as Arc<dyn Any + Send + Sync>;
            if let Ok(arc) = any_model.downcast::<T>() {
                return Ok(arc);
            }

            Err(keyforge_model::error::ForgeError::NotFound(_id.to_string()))
        }

        async fn load_corpus(
            &self,
            _sources: &[keyforge_model::config::CorpusSource],
        ) -> LoaderResult<Arc<Corpus>> {
            Ok(Arc::new(Corpus::default()))
        }
    }

    #[tokio::test]
    async fn test_runtime_end_to_end() {
        let loader = MockLoader;
        let session = SessionBuilder::new(&loader)
            .with_keyboard("test")
            .await
            .unwrap()
            .with_corpus_obj(Arc::new(Corpus::default()))
            .with_cost_matrix(&keyforge_model::config::CostMatrixSource::Predefined(
                "test".into(),
            ))
            .await
            .unwrap()
            .build()
            .unwrap();

        let runtime = Runtime::from(session);
        let layout = Layout::new_unchecked(vec![KeyCode(0)]);

        let analysis = runtime.analyze(&layout).unwrap();
        assert!(analysis.score.is_finite());
    }
}
