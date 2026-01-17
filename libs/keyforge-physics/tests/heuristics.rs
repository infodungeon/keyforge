// libs/keyforge-physics/tests/heuristics.rs

use keyforge_model::{
    Corpus, KeyNode, Keyboard, Layout, Rubric, SearchConfig, CostModel,
    types::{HandIndex, FingerIndex, KeyCode}
};
use keyforge_physics::{identify, suggest_improvements, EngineRequest, ScoringEngine};
use std::sync::Arc;

fn setup_kb() -> Keyboard {
    let keys: Vec<KeyNode> = (0..3).map(|i| KeyNode {
        index: i,
        label: format!("k{}", i),
        hand: HandIndex(0),
        finger: FingerIndex(i as u8),
        x: i as f32,
        ..Default::default()
    }).collect();
    Keyboard::new(keys, 0).unwrap()
}

fn mock_cost_model() -> CostModel {
    let json = r#"{
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
    serde_json::from_str(json).unwrap()
}

#[test]
fn test_fingerprint_identification() {
    let qwerty_str = "qwertyuiopasdfghjkl;zxcvbnm,./";
    let keys: Vec<KeyCode> = qwerty_str.chars().map(|c| KeyCode(c as u16)).collect();
    let layout = Layout::new_unchecked(keys);

    let id = identify(&layout);
    assert!(id.is_some());
    let id = id.unwrap();
    assert_eq!(id.name, "Qwerty");
    assert!(id.similarity > 0.9);
}

#[test]
fn test_heuristics_swap_suggestion_success() {
    let kb = setup_kb();
    let mut corpus = Corpus::default();
    corpus.bigrams.push((0, 2, 1000));

    let mut rubric = Rubric::default();
    rubric.travel_lat = 10.0;

    let layout = Layout::new_unchecked(vec![KeyCode(0), KeyCode(1), KeyCode(2)]);
    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &mock_cost_model()).unwrap();

    let suggestions = engine.suggest_improvements(&layout, false);
    assert!(!suggestions.is_empty(), "Should suggest swapping 0 closer to 2");
    assert!(suggestions[0].improvement_pct > 0.0);
}

#[test]
fn test_heuristics_zero_score_early_return() {
    let kb = setup_kb();
    let corpus = Corpus::default();
    let layout = Layout::new_unchecked(vec![KeyCode(0), KeyCode(1), KeyCode(2)]);
    let engine = ScoringEngine::new(&kb, &corpus, &Rubric::default(), &mock_cost_model()).unwrap();

    let suggestions = engine.suggest_improvements(&layout, false);
    assert!(suggestions.is_empty(), "Zero score should return empty suggestions");
}

#[test]
fn test_public_api_wrappers() {
    let kb = Arc::new(setup_kb());
    let mut corpus = Corpus::default();
    corpus.char_freqs[97] = 100;
    let corpus = Arc::new(corpus);
    let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98), KeyCode(99)]);

    let req = EngineRequest {
        keyboard: kb,
        corpus,
        rubric: Arc::new(Rubric::default()),
        cost_model: Arc::new(mock_cost_model()),
        config: SearchConfig::default(),
        initial_layout: Some(layout),
        pinned_keys: vec![],
    };

    let suggestions = suggest_improvements(&req).unwrap();
    assert!(suggestions.len() <= 5);
}

#[test]
fn test_swap_degradation() {
    let kb = setup_kb();
    let mut corpus = Corpus::default();
    corpus.bigrams.push((0, 1, 1000));
    
    let mut rubric = Rubric::default();
    rubric.travel_lat = 10.0;
    
    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &mock_cost_model()).unwrap();
    
    let layout_keys = vec![KeyCode(0), KeyCode(1), KeyCode(2)];
    let mut pos_map = vec![65535u16; 65536];
    pos_map[0] = 0; pos_map[1] = 1; pos_map[2] = 2;
    
    let delta = engine.calculate_swap_delta(&layout_keys, &pos_map, 1, 2).unwrap();
    
    assert!(delta > 0, "Degrading swap should have positive delta");
}
