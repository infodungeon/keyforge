// libs/keyforge-persistence/tests/persistence_tests.rs

use keyforge_model::config::{Config, CorpusSource};
use keyforge_model::geometry::{KeyboardDefinition, KeyboardGeometry};
use keyforge_model::keycodes::KeycodeRegistry;
use keyforge_model::cost_model::CostModel;
use keyforge_model::Corpus;
use keyforge_persistence::compiler::compile_request;
use keyforge_core::loader::{AssetLoader, LoaderResult};
use keyforge_model::Asset;
use std::sync::Arc;
use std::any::Any;

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
            "models": {},
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
    async fn load<T: Asset>(&self, _id: &str) -> LoaderResult<Arc<T>> {
        let any_kb = Arc::new(KeyboardDefinition::default()) as Arc<dyn Any + Send + Sync>;
        if let Ok(arc) = any_kb.downcast::<T>() {
            if self.fail_kb {
                return Err(keyforge_model::error::ForgeError::NotFound("kb".into()));
            }
            return Ok(arc);
        }

        let json = r#"{ "meta": { "version": "2.0", "description": "Test", "unit": "pts" }, "models": {}, "dynamic_rules": { "sequence_modifiers": {}, "penalties": {}, "constraints": {} } }"#;
        let model: CostModel = serde_json::from_str(json).unwrap();
        let any_model = Arc::new(model) as Arc<dyn Any + Send + Sync>;
        if let Ok(arc) = any_model.downcast::<T>() {
            if self.fail_costs {
                return Err(keyforge_model::error::ForgeError::NotFound("costs".into()));
            }
            return Ok(arc);
        }

        let any_kc = Arc::new(KeycodeRegistry::default()) as Arc<dyn Any + Send + Sync>;
        if let Ok(arc) = any_kc.downcast::<T>() {
            return Ok(arc);
        }

        Err(keyforge_model::error::ForgeError::NotFound(_id.to_string()))
    }

    async fn load_corpus(&self, _sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>> {
        if self.fail_corpus {
            return Err(keyforge_model::error::ForgeError::NotFound("corpus".into()));
        }
        Ok(Arc::new(Corpus::default()))
    }
}

#[tokio::test]
async fn test_compile_request_failures() {
    let config = Config::default();

    let l1 = FailingLoader { fail_kb: true, fail_corpus: false, fail_costs: false };
    assert!(compile_request(&l1, &config, "kb", &[]).await.is_err());
}
