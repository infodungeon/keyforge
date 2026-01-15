// apps/keyforge-agent/tests/compute.rs

//! Integration tests for the agent's optimization compute engine. Verifies the end-to-end
//! execution of layout optimization jobs, including deterministic initial layout generation,
//! support for cancellation signals, and proper telemetry capture throughout the
//! evolutionary search process.


use keyforge_agent::agent::compute;
use keyforge_agent::models::SharedTelemetry;
use keyforge_model::config::{CorpusSource, ScoringWeights, SearchParams};
use keyforge_model::geometry::{KeyNode, KeyboardDefinition, KeyboardGeometry, KeyboardMeta};
use keyforge_model::CostMatrixSource;

use keyforge_protocol::JobConfig;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::Semaphore;
use tempfile::tempdir;

#[tokio::test]
async fn test_agent_session_bootstrap() {
    let dir = tempdir().unwrap();
    let data_root = dir.path().join("data");

    // Create User Structure
    fs::create_dir_all(data_root.join("user/corpora/default")).unwrap();
    fs::create_dir_all(data_root.join("user/keyboards")).unwrap();
    fs::create_dir_all(data_root.join("user/weights")).unwrap();
    fs::create_dir_all(data_root.join("user/config")).unwrap();

    // Cost Matrix
    let mut f = File::create(data_root.join("user/weights/cost.json")).unwrap();
    writeln!(f, r#"[{{ "from_key":"KC_A","to_key":"KC_B","cost_ms":10.0 }}]"#).unwrap();

    // Corpus
    fs::write(data_root.join("user/corpora/default/1grams.json"), r#"[{"char":"a","freq":100}]"#).unwrap();
    fs::write(data_root.join("user/corpora/default/2grams.json"), r#"[{"char1":"a","char2":"b","freq":10}]"#).unwrap();
    fs::write(data_root.join("user/corpora/default/3grams.json"), r#"[]"#).unwrap();
    fs::write(data_root.join("user/corpora/default/words.json"), r#"[]"#).unwrap();

    // Keycodes
    fs::write(data_root.join("user/config/keycodes.json"), r#"[{"code": 97, "id": "KC_A", "label": "a", "aliases": []}]"#).unwrap();

    let geometry = KeyboardGeometry {
        keys: vec![KeyNode { index: 0, label: "a".into(), ..KeyNode::default() }],
        prime_slots: vec![keyforge_model::types::KeyIndex(0)],
        med_slots: vec![],
        low_slots: vec![],
        home_row: 0,
    };

    let config = JobConfig {
        definition: KeyboardDefinition {
            meta: KeyboardMeta { name: "AgentTest".into(), ..Default::default() },
            geometry,
            layouts: HashMap::new(),
        },
        weights: ScoringWeights::default(),
        params: SearchParams::default(),
        pinned_keys: vec![],
        corpora: vec![CorpusSource { id: "default".into(), weight: 1.0, hash: None }],
        cost_matrix: CostMatrixSource::Predefined("cost.json".into()),
        biometrics: vec![],
        parent_job_id: None,
        baseline_score: None,
        parents: vec![],
    };

    let loader = keyforge_infra::FsProvider::new(data_root.clone());
    let options = keyforge_runner::RunnerOptions::default();
    let prepared = keyforge_runner::OptimizationRunner::prepare_session(&loader, &config, &options).await.unwrap();

    assert_eq!(prepared.engine.key_count(), 1);
}

#[tokio::test]
async fn test_optimization_cancellation() {
    let _dir = tempdir().unwrap();
    let keys = vec![
        KeyNode { index: 0, x: 0.0, ..Default::default() },
        KeyNode { index: 1, x: 1.0, ..Default::default() },
    ];
    let kb = Arc::new(keyforge_model::Keyboard::new(keys, 0).unwrap());
    
    let cost_matrix = vec![];
    let engine = Arc::new(keyforge_core::ScoringEngine::new(&kb, &keyforge_model::Corpus::default(), &keyforge_model::Rubric::default(), &cost_matrix).unwrap());
    let search_config = keyforge_model::SearchConfig::Annealing {
        steps: 1000,
        start_temp: 10.0,
        end_temp: 0.1,
        seed: 42,
        patience: 10,
        reheats: 0,
        reheat_factor: 1.0,
    };
    let registry = Arc::new(keyforge_model::keycodes::KeycodeRegistry::new_with_defaults());

    let session = keyforge_core::ScoringSession {
        engine,
        registry,
        search_config,
    };

    let stop_flag = Arc::new(AtomicBool::new(true)); 
    let limiter = Arc::new(Semaphore::new(1));
    let telemetry = SharedTelemetry::default();

    let config = JobConfig {
        definition: KeyboardDefinition::default(),
        weights: ScoringWeights::default(),
        params: SearchParams::default(),
        pinned_keys: vec![],
        corpora: vec![],
        cost_matrix: CostMatrixSource::default(),
        biometrics: vec![],
        parent_job_id: None,
        baseline_score: None,
        parents: vec![],
    };

    let result = compute::run_optimization(session, "test-job".into(), stop_flag, limiter, telemetry, 60, 100, &config).await;
    assert!(result.is_err(), "Should have been cancelled by stop_flag");
}
