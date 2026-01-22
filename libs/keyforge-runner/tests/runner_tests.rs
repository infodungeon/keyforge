use keyforge_core::loader::{AssetLoader, LoaderResult};
use keyforge_core::ProgressCallback;
use keyforge_model::error::ForgeError;
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::keycodes::KeycodeRegistry;
use keyforge_model::Asset;
use keyforge_model::{Corpus, CostModel, KeyCode, KeyIndex};
use keyforge_protocol::JobConfig;
use keyforge_runner::*;
use std::any::Any;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[derive(Debug)]
struct MockLoader;
#[async_trait::async_trait]
impl AssetLoader for MockLoader {
    async fn load<T: Asset>(&self, _id: &str) -> LoaderResult<Arc<T>> {
        let mut kb = KeyboardDefinition::default();
        kb.geometry.keys.push(keyforge_model::KeyNode::default());
        kb.geometry.prime_slots.push(KeyIndex(0));

        let any_kb = Arc::new(kb) as Arc<dyn Any + Send + Sync>;
        if let Ok(arc) = any_kb.downcast::<T>() {
            return Ok(arc);
        }

        let json = r#"{
            "meta": { "version": "2.0", "description": "T", "unit": "pts" },
            "models": { "model_a_row_staggered": { "description": "t", "static_costs": {"universal_hand": {"index": {"base": {"r0": 1.0}}}} } },
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

        Err(ForgeError::NotFound(_id.to_string()))
    }
    async fn load_corpus(
        &self,
        _sources: &[keyforge_model::config::CorpusSource],
    ) -> LoaderResult<Arc<Corpus>> {
        Ok(Arc::new(Corpus::default()))
    }
}

fn test_search_config() -> keyforge_model::SearchConfig {
    keyforge_model::SearchConfig::Annealing {
        steps: 100,
        start_temp: 10.0,
        end_temp: 0.1,
        seed: 42,
        patience: 10,
        reheats: 0,
        reheat_factor: 0.5,
        include_thumbs: false,
    }
}

#[tokio::test]
async fn test_runner_lifecycle() {
    struct NoOpCallback;
    impl ProgressCallback for NoOpCallback {
        fn on_progress(
            &self,
            _epoch: usize,
            _score: f32,
            _layout: &[KeyCode],
            _ips: f32,
        ) -> keyforge_core::OptimizationControl {
            keyforge_core::OptimizationControl::Continue
        }
    }

    let loader = MockLoader;
    let mut config = JobConfig::default();
    config
        .params
        .params
        .insert("search_steps".to_string(), 100.0); // Reduce steps via params
    config
        .definition
        .geometry
        .keys
        .push(keyforge_model::KeyNode::default());
    config.definition.geometry.prime_slots.push(KeyIndex(0));

    let options = RunnerOptions {
        keycodes_file: "kc".into(),
        ..Default::default()
    };

    let session = OptimizationRunner::prepare_session(&loader, &config, &options)
        .await
        .unwrap();
    assert_eq!(session.engine.key_count(), 1);

    let stop = Arc::new(AtomicBool::new(false));
    let res = OptimizationRunner::run(session, "job".into(), stop, NoOpCallback, options, &config)
        .await
        .unwrap();
    assert!(res.score >= 0.0);
}

#[tokio::test]
async fn test_runner_prepare_job() {
    let loader = MockLoader;
    let runner = Runner::new(&loader);
    let mut config = JobConfig::default();
    config
        .definition
        .geometry
        .keys
        .push(keyforge_model::KeyNode::default());
    config.definition.geometry.prime_slots.push(KeyIndex(0));

    let rt = runner.prepare_job(&config, None).await.unwrap();
    assert_eq!(rt.engine.key_count(), 1);
}

#[tokio::test]
async fn test_runner_pinned_keys() {
    struct NoOpCallback;
    impl ProgressCallback for NoOpCallback {
        fn on_progress(
            &self,
            _epoch: usize,
            _score: f32,
            _layout: &[KeyCode],
            _ips: f32,
        ) -> keyforge_core::OptimizationControl {
            keyforge_core::OptimizationControl::Continue
        }
    }

    let _loader = MockLoader;
    let mut config = JobConfig::default();
    config
        .params
        .params
        .insert("search_steps".to_string(), 100.0);
    config.pinned_keys.push(keyforge_model::KeyConstraint {
        index: KeyIndex(0),
        key: "SPACE".to_string(),
    });

    let registry = KeycodeRegistry::new(vec![keyforge_model::keycodes::KeycodeDefinition {
        code: KeyCode(0),
        id: "SPACE".into(),
        label: " ".into(),
        aliases: vec![],
    }]);

    let mut cm = CostModel::default();
    let mut fingers = std::collections::HashMap::new();
    fingers.insert(
        "index".to_string(),
        keyforge_model::cost_model::FingerDefinition::Standard(std::collections::HashMap::from([
            (
                "base".to_string(),
                std::collections::HashMap::from([("r0".to_string(), 1.0)]),
            ),
        ])),
    );
    cm.models.insert(
        "model_a_row_staggered".into(),
        keyforge_model::cost_model::ModelDefinition {
            description: "test".into(),
            static_costs: std::collections::HashMap::from([(
                "universal_hand".to_string(),
                keyforge_model::cost_model::HandDefinition { fingers },
            )]),
        },
    );

    let session = keyforge_core::ScoringSession {
        engine: keyforge_physics::EngineFactory::new_exact(
            &keyforge_model::Keyboard::new(
                vec![keyforge_model::KeyNode::default()],
                0,
                "test".into(),
            )
            .unwrap(),
            &Corpus::default(),
            &keyforge_model::Rubric::default(),
            &cm,
        )
        .unwrap()
        .into(),
        registry: Arc::new(registry),
        search_config: test_search_config(),
    };

    let stop = Arc::new(AtomicBool::new(false));
    let options = RunnerOptions::default();
    let res = OptimizationRunner::run(session, "job".into(), stop, NoOpCallback, options, &config)
        .await
        .unwrap();
    assert!(res.score >= 0.0);
}
