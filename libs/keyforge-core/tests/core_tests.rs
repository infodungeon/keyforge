// libs/keyforge-core/tests/core_tests.rs

use keyforge_core::*;
use keyforge_model::{Corpus, KeyNode, Keyboard, Layout, Rubric, SearchConfig, CostModel, types::KeyCode};
use keyforge_physics::EngineRequest;
use std::sync::Arc;

fn minimal_keyboard() -> Keyboard {
    Keyboard::new(
        vec![
            KeyNode {
                index: 0,
                label: "Q".to_string(),
                hand: keyforge_model::types::HandIndex(0),
                finger: keyforge_model::types::FingerIndex(0),
                row: keyforge_model::types::RowIndex(0),
                col: keyforge_model::types::ColIndex(0),
                x: 0.0,
                y: 0.0,
                is_home: false,
                ..Default::default()
            },
            KeyNode {
                index: 1,
                label: "A".to_string(),
                hand: keyforge_model::types::HandIndex(0),
                finger: keyforge_model::types::FingerIndex(0),
                row: keyforge_model::types::RowIndex(1),
                col: keyforge_model::types::ColIndex(0),
                x: 0.0,
                y: 1.0,
                is_home: true,
                ..Default::default()
            },
        ],
        1,
    ).unwrap()
}

fn minimal_corpus() -> Corpus {
    let mut c = Corpus::default();
    c.char_freqs[0] = 100;
    c.char_freqs[1] = 50;
    c
}

fn minimal_rubric() -> Rubric {
    Rubric {
        sfb_base: 100.0,
        sfb_lateral: 50.0,
        travel_lat: 2.0,
        travel_vert: 3.0,
        finger_effort: [1.0; 5],
        redirect: 200.0,
        roll_bonus: 10.0,
        trigram_coverage: 0.5,
        trigram_limit: 1000,
        ..Default::default()
    }
}

fn minimal_cost_model() -> CostModel {
    let json = r#"{
        "meta": { "version": "2.0", "description": "Test", "unit": "pts" },
        "models": {
            "model_a_row_staggered": {
                "description": "Test Model",
                "static_costs": {
                    "universal_hand": {
                        "thumb": { "pos_1": 100.0 },
                        "index": { "base": { "r0": 100.0, "r1": 100.0 } },
                        "middle": { "base": { "r0": 100.0, "r1": 100.0 } },
                        "ring": { "base": { "r0": 100.0, "r1": 100.0 } },
                        "pinky": { "base": { "r0": 100.0, "r1": 100.0 } }
                    }
                }
            }
        },
        "dynamic_rules": { "sequence_modifiers": {}, "penalties": {}, "constraints": {} }
    }"#;
    serde_json::from_str(json).unwrap()
}

fn minimal_request() -> EngineRequest {
    EngineRequest {
        keyboard: Arc::new(minimal_keyboard()),
        corpus: Arc::new(minimal_corpus()),
        rubric: Arc::new(minimal_rubric()),
        cost_model: Arc::new(minimal_cost_model()),
        config: SearchConfig::Annealing {
            steps: 5,
            start_temp: 10.0,
            end_temp: 0.1,
            seed: 42,
            patience: 10,
            reheats: 0,
            reheat_factor: 0.5,
            include_thumbs: false,
        },
        initial_layout: Some(Layout::new_unchecked(vec![KeyCode(0), KeyCode(1)])),
        pinned_keys: vec![],
    }
}

struct TestCallback;
impl ProgressCallback for TestCallback {
    fn on_progress(&self, _epoch: usize, _score: f32, _layout: &[KeyCode], _ips: f32) -> bool {
        true
    }
}

#[test]
fn test_build_engine() {
    let req = minimal_request();
    let engine = build_engine(&req).unwrap();
    assert!(engine.key_count() >= 2);
}

#[test]
fn test_analyze_with_engine() {
    let req = minimal_request();
    let engine = build_engine(&req).unwrap();
    let layout = Layout::new_unchecked(vec![KeyCode(0), KeyCode(1)]);

    let report = analyze_with_engine(&engine, &layout).unwrap();
    assert!(report.score.is_finite());
}

#[test]
fn test_score_with_engine() {
    let req = minimal_request();
    let engine = build_engine(&req).unwrap();
    let layout = Layout::new_unchecked(vec![KeyCode(0), KeyCode(1)]);

    let score = score_with_engine(&engine, &layout).unwrap();
    assert!(score.is_finite());
}

#[test]
fn test_suggest_with_engine() {
    let req = minimal_request();
    let engine = build_engine(&req).unwrap();
    let layout = Layout::new_unchecked(vec![KeyCode(0), KeyCode(1)]);

    let suggestions = suggest_with_engine(&engine, &layout).unwrap();
    let _ = suggestions.len();
}

#[test]
fn test_analyze_legacy() {
    let req = minimal_request();
    let report = analyze(&req).unwrap();
    assert!(report.score.is_finite());
}

#[test]
fn test_score_legacy() {
    let req = minimal_request();
    let result = score(&req).unwrap();
    assert!(result.score.is_finite());
}

#[test]
fn test_suggest_legacy() {
    let req = minimal_request();
    let suggestions = suggest(&req).unwrap();
    let _ = suggestions.len();
}

#[test]
fn test_identify() {
    let layout = Layout::new_unchecked(vec![KeyCode(0), KeyCode(1), KeyCode(2), KeyCode(3), KeyCode(4)]);
    let identity = identify(&layout);
    let _id = identity;
}

#[test]
fn test_optimize_legacy() {
    let mut req = minimal_request();
    req.config = SearchConfig::Annealing {
        steps: 10,
        start_temp: 1.0,
        end_temp: 0.1,
        seed: 42,
        patience: 5,
        reheats: 0,
        reheat_factor: 0.5,
        include_thumbs: false,
    };

    let result = optimize(&req).unwrap();
    assert!(result.score.is_finite());
    assert!(result.layout.keys.len() >= 2);
}

#[test]
fn test_optimize_with_callback() {
    let mut req = minimal_request();
    req.config = SearchConfig::Annealing {
        steps: 10,
        start_temp: 1.0,
        end_temp: 0.1,
        seed: 42,
        patience: 5,
        reheats: 0,
        reheat_factor: 0.5,
        include_thumbs: false,
    };

    let result = optimize_with_callback(&req, TestCallback).unwrap();
    assert!(result.score.is_finite());
}

#[test]
fn test_optimize_with_engine() {
    let req = minimal_request();
    let engine = build_engine(&req).unwrap();
    let engine_arc = Arc::new(engine);

    let config = SearchConfig::Annealing {
        steps: 10,
        start_temp: 1.0,
        end_temp: 0.1,
        seed: 42,
        patience: 5,
        reheats: 0,
        reheat_factor: 0.5,
        include_thumbs: false,
    };

    let result = optimize_with_engine(engine_arc, &config, TestCallback, None, None).unwrap();
    assert!(result.score.is_finite());
}
