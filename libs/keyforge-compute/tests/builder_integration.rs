use keyforge_compute::*;
use keyforge_model::{KeyNode, Keyboard, Corpus, Rubric, CostModel, Layout, SearchConfig, Asset};
use keyforge_physics::EngineFactory;
use keyforge_model::keycodes::KeycodeRegistry;
use std::sync::Arc;
use keyforge_core::{ProgressCallback, ScoringSession};
use keyforge_core::loader::{AssetLoader, LoaderResult};
use keyforge_model::config::{CorpusSource, CostMatrixSource};
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_protocol::BiometricSample;
use std::any::Any;

#[derive(Debug)]
struct MockLoader;
#[async_trait::async_trait]
impl AssetLoader for MockLoader {
    async fn load<T: Asset>(&self, id: &str) -> LoaderResult<Arc<T>> {
        if id == "missing" { return Err(keyforge_model::error::ForgeError::NotFound(id.into())); }
        
        let any_kb = Arc::new(KeyboardDefinition {
            geometry: keyforge_model::geometry::KeyboardGeometry {
                keys: vec![keyforge_model::KeyNode::default()],
                prime_slots: vec![keyforge_model::types::KeyIndex(0)],
                ..Default::default()
            },
            ..Default::default()
        }) as Arc<dyn Any + Send + Sync>;
        if let Ok(arc) = any_kb.downcast::<T>() { return Ok(arc); }

        let json = r#"{
            "meta": { "version": "2.0", "description": "T", "unit": "pts" },
            "models": { "model_a_row_staggered": { "description": "t", "static_costs": {} } },
            "dynamic_rules": { "sequence_modifiers": {}, "penalties": {}, "constraints": {} }
        }"#;
        let model: CostModel = serde_json::from_str(json).unwrap();
        let any_model = Arc::new(model) as Arc<dyn Any + Send + Sync>;
        if let Ok(arc) = any_model.downcast::<T>() { return Ok(arc); }

        let any_kc = Arc::new(KeycodeRegistry::default()) as Arc<dyn Any + Send + Sync>;
        if let Ok(arc) = any_kc.downcast::<T>() { return Ok(arc); }

        Err(keyforge_model::error::ForgeError::NotFound(id.to_string()))
    }
    async fn load_corpus(&self, _sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>> {
        Ok(Arc::new(Corpus::default()))
    }
}

fn setup_runtime() -> Runtime {
    let kb = Keyboard::new(vec![KeyNode::default()], 0).unwrap();
    let mut cm = CostModel::default();
    cm.models.insert("model_a_row_staggered".into(), keyforge_model::cost_model::ModelDefinition {
        description: "test".into(),
        static_costs: std::collections::HashMap::new(),
    });
    let engine = EngineFactory::new_exact(&kb, &Corpus::default(), &Rubric::default(), &cm).unwrap();
    let registry = Arc::new(KeycodeRegistry::new_with_defaults());
    Runtime::new(Arc::from(engine), registry, SearchConfig::default())
}

#[test]
fn test_runtime_methods() {
    struct NoOpCallback;
    impl ProgressCallback for NoOpCallback {
        fn on_progress(&self, _epoch: usize, _score: f32, _layout: &[keyforge_model::KeyCode], _ips: f32) -> bool { true }
    }

    let mut rt = setup_runtime();
    let layout = Layout::new_unchecked(vec![keyforge_model::KeyCode(0)]);
    
    assert!(rt.score(&layout).is_ok());
    assert!(rt.analyze(&layout).is_ok());
    assert!(rt.suggest_improvements(&layout).is_ok());
    
    // Trigger include_thumbs branch
    rt.search_config = SearchConfig::Annealing { 
        steps: 10, start_temp: 1.0, end_temp: 0.1, seed: 0,
        patience: 10, reheats: 0, reheat_factor: 0.5, include_thumbs: true 
    };
    assert!(rt.suggest_improvements(&layout).is_ok());

    assert!(rt.optimize(NoOpCallback, None, None).is_ok());
}

#[test]
fn test_runtime_from_session() {
    let kb = Keyboard::new(vec![KeyNode::default()], 0).unwrap();
    let mut cm = CostModel::default();
    cm.models.insert("model_a_row_staggered".into(), keyforge_model::cost_model::ModelDefinition {
        description: "test".into(),
        static_costs: std::collections::HashMap::new(),
    });
    let engine = EngineFactory::new_exact(&kb, &Corpus::default(), &Rubric::default(), &cm).unwrap();
    let session = ScoringSession::new(Arc::from(engine), Arc::new(KeycodeRegistry::default()), SearchConfig::default());
    
    let rt = Runtime::from(session);
    assert_eq!(rt.registry.definitions.len(), 0);
}

#[tokio::test]
async fn test_session_builder_lifecycle() {
    let loader = MockLoader;
    let kb_def = Arc::new(KeyboardDefinition {
        geometry: keyforge_model::geometry::KeyboardGeometry {
            keys: vec![keyforge_model::KeyNode::default()],
            prime_slots: vec![keyforge_model::types::KeyIndex(0)],
            ..Default::default()
        },
        ..Default::default()
    });
    let corp = Arc::new(Corpus::default());
    let mut cm = CostModel::default();
    cm.models.insert("model_a_row_staggered".into(), keyforge_model::cost_model::ModelDefinition {
        description: "test".into(),
        static_costs: std::collections::HashMap::new(),
    });
    let cm_arc = Arc::new(cm);

    let builder = SessionBuilder::new(&loader)
        .with_keyboard("kb").await.unwrap()
        .with_keyboard_def(kb_def)
        .with_corpus(&[]).await.unwrap()
        .with_corpus_obj(corp)
        .with_cost_matrix(&CostMatrixSource::Predefined("cm".into())).await.unwrap()
        .with_cost_model_obj(cm_arc)
        .with_keycodes("kc").await.unwrap()
        .with_rubric(Rubric::default())
        .with_config(SearchConfig::default())
        .with_biometrics(vec![BiometricSample { bigram: "th".into(), ms: 100.0, timestamp: 0 }]);
    
    let debug_str = format!("{:?}", builder);
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
    let b2 = SessionBuilder::new(&loader).with_keyboard("kb").await.unwrap();
    assert!(b2.build().is_err());

    // 3. Missing cost model
    let b3 = SessionBuilder::new(&loader).with_keyboard("kb").await.unwrap().with_corpus(&[]).await.unwrap();
    assert!(b3.build().is_err());

    // 4. Default registry and rubric
    let mut cm = CostModel::default();
    cm.models.insert("model_a_row_staggered".into(), keyforge_model::cost_model::ModelDefinition {
        description: "test".into(), static_costs: std::collections::HashMap::new(),
    });
    let b4 = SessionBuilder::new(&loader)
        .with_keyboard_def(Arc::new(KeyboardDefinition {
            geometry: keyforge_model::geometry::KeyboardGeometry {
                keys: vec![keyforge_model::KeyNode::default()],
                prime_slots: vec![keyforge_model::types::KeyIndex(0)],
                ..Default::default()
            }, ..Default::default()
        }))
        .with_corpus_obj(Arc::new(Corpus::default()))
        .with_cost_model_obj(Arc::new(cm));
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
        }, ..Default::default()
    });
    
    let builder = SessionBuilder::new(&loader).with_keyboard_def(kb_def);
    let res = builder.build();
    assert!(matches!(res, Err(keyforge_model::error::ForgeError::Config(_))));
}
