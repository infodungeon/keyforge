use keyforge_core::*;
use keyforge_model::{Corpus, KeyNode, Keyboard, Layout, Rubric, SearchConfig};
use keyforge_physics::EngineRequest;
use std::sync::Arc;

fn minimal_keyboard() -> Keyboard {
    Keyboard::new(
        vec![
            KeyNode {
                id: 0,
                label: "Q".to_string(),
                hand: 0,
                finger: 0,
                row: 0,
                col: 0,
                x: 0.0,
                y: 0.0,
                is_home: false,
            },
            KeyNode {
                id: 1,
                label: "A".to_string(),
                hand: 0,
                finger: 0,
                row: 1,
                col: 0,
                x: 0.0,
                y: 1.0,
                is_home: true,
            },
        ],
        1,
    )
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
    }
}

fn minimal_request() -> EngineRequest {
    EngineRequest {
        keyboard: Arc::new(minimal_keyboard()),
        corpus: Arc::new(minimal_corpus()),
        rubric: Arc::new(minimal_rubric()),
        cost_overrides: vec![],
        config: SearchConfig::Annealing {
            steps: 100,
            start_temp: 10.0,
            end_temp: 0.1,
            seed: 42,
            patience: 10,
            reheats: 0,
            reheat_factor: 0.5,
        },
        initial_layout: Some(Layout::new_unchecked(vec![0, 1])),
        pinned_keys: vec![],
    }
}

struct TestCallback;
impl ProgressCallback for TestCallback {
    fn on_progress(&self, _epoch: usize, _score: f32, _layout: &[u16], _ips: f32) -> bool {
        true
    }
}

#[test]
fn test_build_engine() {
    let req = minimal_request();
    let engine = build_engine(&req);
    assert!(engine.key_count() >= 2);
}

#[test]
fn test_analyze_with_engine() {
    let req = minimal_request();
    let engine = build_engine(&req);
    let layout = Layout::new_unchecked(vec![0, 1]);

    let report = analyze_with_engine(&engine, &layout);
    assert!(report.score.is_finite());
}

#[test]
fn test_score_with_engine() {
    let req = minimal_request();
    let engine = build_engine(&req);
    let layout = Layout::new_unchecked(vec![0, 1]);

    let score = score_with_engine(&engine, &layout);
    assert!(score.is_finite());
}

#[test]
fn test_suggest_with_engine() {
    let req = minimal_request();
    let engine = build_engine(&req);
    let layout = Layout::new_unchecked(vec![0, 1]);

    let suggestions = suggest_with_engine(&engine, &layout);
    let _ = suggestions.len();
}

#[test]
fn test_analyze_legacy() {
    let req = minimal_request();
    let report = analyze(&req);
    assert!(report.score.is_finite());
}

#[test]
fn test_score_legacy() {
    let req = minimal_request();
    let result = score(&req);
    assert!(result.score.is_finite());
}

#[test]
fn test_suggest_legacy() {
    let req = minimal_request();
    let suggestions = suggest(&req);
    let _ = suggestions.len();
}

#[test]
fn test_identify() {
    let layout = Layout::new_unchecked(vec![0, 1, 2, 3, 4]);
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
    };

    let result = optimize(&req);
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
    };

    let result = optimize_with_callback(&req, TestCallback);
    assert!(result.score.is_finite());
}

#[test]
fn test_optimize_with_engine() {
    let req = minimal_request();
    let engine = build_engine(&req);
    let engine_arc = Arc::new(engine);

    let config = SearchConfig::Annealing {
        steps: 10,
        start_temp: 1.0,
        end_temp: 0.1,
        seed: 42,
        patience: 5,
        reheats: 0,
        reheat_factor: 0.5,
    };

    let result = optimize_with_engine(engine_arc, &config, TestCallback);
    assert!(result.score.is_finite());
}
