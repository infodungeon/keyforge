// libs/keyforge-runner/tests/runner_integration.rs

use async_trait::async_trait;
use keyforge_core::loader::{AssetLoader, LoaderResult};
use keyforge_model::{Asset, Corpus, KeyNode, KeyboardDefinition};
use keyforge_protocol::JobConfig;
use keyforge_runner::{OptimizationRunner, RunnerOptions};
use std::any::Any;
use std::sync::Arc;

#[derive(Debug)]
struct MockLoader;

#[async_trait]
impl AssetLoader for MockLoader {
    async fn load<T: Asset>(&self, _id: &str) -> LoaderResult<Arc<T>> {
        let any_kb = Arc::new(KeyboardDefinition {
            meta: Default::default(),
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
                    "static_costs": {
                        "universal_hand": {
                            "index": {
                                "base": { "r0": 1.0 }
                            }
                        }
                    }
                }
            },
            "dynamic_rules": { "sequence_modifiers": {}, "penalties": {}, "constraints": {} }
        }"#;
        let model: keyforge_model::cost_model::CostModel = serde_json::from_str(json).unwrap();
        let any_model = Arc::new(model) as Arc<dyn Any + Send + Sync>;
        if let Ok(arc) = any_model.downcast::<T>() {
            return Ok(arc);
        }

        let any_kc = Arc::new(keyforge_model::keycodes::KeycodeRegistry::default())
            as Arc<dyn Any + Send + Sync>;
        if let Ok(arc) = any_kc.downcast::<T>() {
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
async fn test_runner_prepare_session() {
    let loader = MockLoader;
    let mut config = JobConfig::default();
    // JobConfig::default() has empty geometry, which overrides the loader's default if we aren't careful
    // but prepare_session uses config.definition.geometry.
    config.definition.geometry.keys.push(KeyNode {
        index: 0,
        x: 0.0,
        y: 0.0,
        ..Default::default()
    });
    config.definition.geometry.home_row = 0;
    config
        .definition
        .geometry
        .prime_slots
        .push(keyforge_model::KeyIndex(0));

    let options = RunnerOptions::default();

    let session = OptimizationRunner::prepare_session(&loader, &config, &options).await;
    assert!(
        session.is_ok(),
        "Failed to prepare runner session: {:?}",
        session.err()
    );
}
