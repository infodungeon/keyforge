// libs/keyforge-physics/tests/robustness.rs

use keyforge_model::{
    Corpus, KeyNode, Keyboard, Layout, Rubric, CostModel,
    types::{HandIndex, FingerIndex, KeyCode}
};
use keyforge_physics::ScoringEngine;

fn setup_kb() -> Keyboard {
    let keys: Vec<KeyNode> = (0..5).map(|i| KeyNode {
        index: i,
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
fn test_math_boundaries_infinity() {
    let kb = setup_kb();
    let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98), KeyCode(99), KeyCode(100), KeyCode(101)]);
    let mut corpus = Corpus::default();
    corpus.bigrams.push((97, 98, 1000));

    let rubric = Rubric {
        travel_lat: f32::INFINITY,
        ..Rubric::default()
    };

    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &mock_cost_model()).unwrap();
    let score = engine.score(&layout).unwrap();

    assert!(score > 1_000_000.0);
    assert!(score.is_finite());
}

#[test]
fn test_math_boundaries_nan() {
    let kb = setup_kb();
    let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98), KeyCode(99), KeyCode(100), KeyCode(101)]);
    let mut corpus = Corpus::default();
    corpus.bigrams.push((97, 98, 1000));

    let rubric = Rubric {
        travel_lat: f32::NAN,
        ..Rubric::default()
    };

    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &mock_cost_model()).unwrap();
    let score = engine.score(&layout).unwrap();

    assert!(score >= 0.0);
    assert!(!score.is_nan());
}

#[test]
fn test_saturation_protection() {
    let kb = setup_kb();
    let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98), KeyCode(99), KeyCode(100), KeyCode(101)]);
    let mut corpus = Corpus::default();
    corpus.bigrams.push((97, 98, u32::MAX));

    let rubric = Rubric {
        travel_lat: 1_000_000.0,
        ..Rubric::default()
    };

    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &mock_cost_model()).unwrap();
    let score = engine.score(&layout).unwrap();
    assert!(score.is_finite());
}

#[test]
fn test_missing_keys_in_layout() {
    let kb = setup_kb();
    let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(0), KeyCode(99), KeyCode(100), KeyCode(101)]);
    let mut corpus = Corpus::default();
    corpus.bigrams.push((97, 98, 100)); 

    let engine = ScoringEngine::new(&kb, &corpus, &Rubric::default(), &mock_cost_model()).unwrap();
    let score = engine.score(&layout).unwrap();
    assert_eq!(score, 0.0);
}

#[test]
fn test_swap_delta_bounds() {
    let kb = setup_kb();
    let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98), KeyCode(99), KeyCode(100), KeyCode(101)]);
    let mut corpus = Corpus::default();
    corpus.bigrams.push((97, 98, 100));

    let engine = ScoringEngine::new(&kb, &corpus, &Rubric::default(), &mock_cost_model()).unwrap();
    let mut pos_map = vec![65535u16; 65536];
    for (i, &code) in layout.keys.iter().enumerate() {
        pos_map[code.0 as usize] = i as u16;
    }

    let delta = engine.calculate_swap_delta(&layout.keys, &pos_map, 0, 100).unwrap();
    assert_eq!(delta, 0);
}

#[test]
fn test_analyze_layout_empty() {
    let kb = setup_kb();
    let corpus = Corpus::default();
    let rubric = Rubric::default();
    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &mock_cost_model()).unwrap();
    
    let layout = Layout::new_unchecked(vec![]);
    let result = engine.analyze(&layout);
    assert!(result.is_err());
}

#[test]
fn test_compiler_trigram_pruning() {
    let kb = setup_kb();
    let mut corpus = Corpus::default();
    for i in 0..20 {
        corpus.trigrams.push((0, 1, i as u16, 100));
    }
    
    let mut rubric = Rubric::default();
    rubric.trigram_limit = 5;
    
    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &mock_cost_model()).unwrap();
    
    assert_eq!(engine.trigram_count(), 5);
}

#[test]
fn test_finger_origin_fallback() {
    let keys = vec![
        KeyNode { index: 0, finger: FingerIndex(1), is_home: false, ..Default::default() }
    ];
    let kb = Keyboard::new(keys, 0).unwrap();
    let corpus = Corpus::default();
    let rubric = Rubric::default();
    
    let result = ScoringEngine::new(&kb, &corpus, &rubric, &mock_cost_model());
    assert!(result.is_ok());
}
