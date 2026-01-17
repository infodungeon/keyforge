// apps/keyforge-agent/tests/compute.rs

use keyforge_agent::agent::compute::run_optimization;
use keyforge_agent::models::AgentTelemetry;
use keyforge_protocol::JobConfig;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::Semaphore;
use keyforge_model::cost_model::CostModel;
use keyforge_model::{Keyboard, KeyNode, KeyIndex, KeycodeRegistry, SearchConfig, ScoringWeights, SearchParams, CostMatrixSource};

#[tokio::test]
async fn test_compute_optimization_run() {
    // Mock Geometry
    let kb_def = keyforge_model::KeyboardDefinition {
        geometry: keyforge_model::KeyboardGeometry {
            keys: vec![KeyNode::default()],
            prime_slots: vec![KeyIndex(0)],
            med_slots: vec![],
            low_slots: vec![],
            home_row: 0,
        },
        ..Default::default()
    };

    let kb = Keyboard::new(
        kb_def.geometry.keys.clone(),
        kb_def.geometry.home_row,
    ).unwrap();
    
    // Create a valid CostModel
    let cost_json = r#"{
        "meta": { "version": "2.0", "description": "Test", "unit": "pts" },
        "models": {
            "model_a_row_staggered": {
                "description": "Test Model",
                "static_costs": {
                    "universal_hand": {
                        "thumb": { "pos_1": 100.0 },
                        "index": { "base": { "r0": 100.0 } },
                        "middle": { "base": { "r0": 100.0 } },
                        "ring": { "base": { "r0": 100.0 } },
                        "pinky": { "base": { "r0": 100.0 } }
                    }
                }
            }
        },
        "dynamic_rules": { "sequence_modifiers": {}, "penalties": {}, "constraints": {} }
    }"#;
    let cost_model: CostModel = serde_json::from_str(cost_json).unwrap();

    let engine = Arc::new(keyforge_core::ScoringEngine::new(
        &kb, 
        &keyforge_model::Corpus::default(), 
        &keyforge_model::Rubric::default(), 
        &cost_model
    ).unwrap());
    
    let search_config = SearchConfig::Annealing {
        steps: 100,
        start_temp: 10.0,
        end_temp: 0.1,
        seed: 42,
        patience: 10,
        reheats: 0,
        reheat_factor: 0.5,
        include_thumbs: false,
    };

    let session = keyforge_core::ScoringSession::new(
        engine,
        Arc::new(KeycodeRegistry::default()),
        search_config,
    );

    let job_config = JobConfig {
        definition: kb_def,
        weights: ScoringWeights::default(),
        params: SearchParams::default(),
        pinned_keys: vec![],
        corpora: vec![],
        cost_matrix: CostMatrixSource::Predefined("test".to_string()),
        biometrics: vec![],
        parent_job_id: None,
        baseline_score: None,
        parents: vec![],
    };

    let stop_flag = Arc::new(AtomicBool::new(false));
    let limiter = Arc::new(Semaphore::new(1));
    let telemetry = Arc::new(AgentTelemetry::default());

    // Start optimization
    let result = run_optimization(
        session,
        "job-123".to_string(),
        stop_flag,
        limiter,
        telemetry,
        3600,
        100,
        &job_config
    ).await.expect("Optimization should complete successfully");

    // Verify result
    assert!(result.score >= 0.0);
}