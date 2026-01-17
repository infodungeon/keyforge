// libs/keyforge-persistence/tests/persistence_tests.rs

use keyforge_model::config::{Config, CorpusSource};
use keyforge_model::geometry::{KeyboardDefinition, KeyboardGeometry};
use keyforge_model::keycodes::KeycodeRegistry;
use keyforge_model::cost_model::CostModel;
use keyforge_model::Corpus;
use keyforge_persistence::compiler::compile_request;
use keyforge_core::loader::{AssetLoader, LoaderResult};
use std::sync::Arc;

#[derive(Debug)]
struct MockLoader;

#[async_trait::async_trait]
impl AssetLoader for MockLoader {
    async fn load_keyboard(&self, _name: &str) -> LoaderResult<Arc<KeyboardDefinition>> {
        Ok(Arc::new(KeyboardDefinition {
            meta: Default::default(),
            geometry: KeyboardGeometry {
                keys: vec![Default::default()],
                prime_slots: vec![],
                med_slots: vec![],
                low_slots: vec![],
                home_row: 0,
            },
            layouts: Default::default(),
        }))
    }

    async fn load_corpus(&self, _sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>> {
        Ok(Arc::new(Corpus::default()))
    }

    async fn load_cost_model(&self, _filename: &str) -> LoaderResult<Arc<CostModel>> {
        // Return a minimal valid CostModel
        let json = r#"{
            "meta": { "version": "2.0", "description": "Test", "unit": "pts" },
            "models": {},
            "dynamic_rules": { "sequence_modifiers": {}, "penalties": {}, "constraints": {} }
        }"#;
        let model: CostModel = serde_json::from_str(json).unwrap();
        Ok(Arc::new(model))
    }

    async fn load_keycodes(&self, _filename: &str) -> LoaderResult<Arc<KeycodeRegistry>> {
        Ok(Arc::new(KeycodeRegistry::default()))
    }
}

#[tokio::test]
async fn test_compile_request_success() {
    let loader = MockLoader;
    let config = Config::default();
    let res = compile_request(&loader, &config, "test_kb", &[]).await;
    assert!(res.is_ok());
}

#[derive(Debug)]
struct FailingLoader {
    fail_kb: bool,
    fail_corpus: bool,
    fail_costs: bool,
}

#[async_trait::async_trait]
impl AssetLoader for FailingLoader {
    async fn load_keyboard(&self, _name: &str) -> LoaderResult<Arc<KeyboardDefinition>> {
        if self.fail_kb {
            return Err(keyforge_model::error::ForgeError::NotFound("kb".into()));
        }
        Ok(Arc::new(KeyboardDefinition::default()))
    }

    async fn load_corpus(&self, _sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>> {
        if self.fail_corpus {
            return Err(keyforge_model::error::ForgeError::NotFound("corpus".into()));
        }
        Ok(Arc::new(Corpus::default()))
    }

    async fn load_cost_model(&self, _filename: &str) -> LoaderResult<Arc<CostModel>> {
        if self.fail_costs {
            return Err(keyforge_model::error::ForgeError::NotFound("costs".into()));
        }
        let json = r#"{ "meta": { "version": "2.0", "description": "Test", "unit": "pts" }, "models": {}, "dynamic_rules": { "sequence_modifiers": {}, "penalties": {}, "constraints": {} } }"#;
        let model: CostModel = serde_json::from_str(json).unwrap();
        Ok(Arc::new(model))
    }

    async fn load_keycodes(&self, _filename: &str) -> LoaderResult<Arc<KeycodeRegistry>> {
        Ok(Arc::new(KeycodeRegistry::default()))
    }
}

#[tokio::test]
async fn test_compile_request_failures() {
    let config = Config::default();

    let l1 = FailingLoader { fail_kb: true, fail_corpus: false, fail_costs: false };
    assert!(compile_request(&l1, &config, "kb", &[]).await.is_err());

    // Note: compile_request might fail fast, so we test one by one or check error types if needed.
}
