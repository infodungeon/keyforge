// libs/keyforge-physics/tests/heuristics.rs

//! Integration tests for the physics engine's heuristic reasoning. Verifies the accuracy
//! of standard layout identification (e.g., Qwerty) and validates that the engine's
//! improvement suggestions correctly identify score-reducing key swaps based on corpus
//! frequencies and geometric travel costs.


use keyforge_model::{
    Corpus, KeyNode, Keyboard, Layout, Rubric, SearchConfig, 
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
    // Bigram (0, 2) -> High Freq. Char 0 wants to be close to Char 2.
    corpus.bigrams.push((0, 2, 1000));

    let mut rubric = Rubric::default();
    rubric.travel_lat = 10.0;

    // Layout: 0, 1, 2. 
    // Char 0 is at x=0. Char 2 is at x=2. Distance = 2.
    // Swapping 0 and 1 puts Char 0 at x=1. Distance = 1. (Improvement)
    let layout = Layout::new_unchecked(vec![KeyCode(0), KeyCode(1), KeyCode(2)]);
    let cost_matrix = vec![];
    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &cost_matrix).unwrap();

    let suggestions = engine.suggest_improvements(&layout);
    assert!(!suggestions.is_empty(), "Should suggest swapping 0 closer to 2");
    assert!(suggestions[0].improvement_pct > 0.0);
}

#[test]
fn test_heuristics_zero_score_early_return() {
    let kb = setup_kb();
    let corpus = Corpus::default(); // Empty corpus = 0 score
    let layout = Layout::new_unchecked(vec![KeyCode(0), KeyCode(1), KeyCode(2)]);
    let cost_matrix = vec![];
    let engine = ScoringEngine::new(&kb, &corpus, &Rubric::default(), &cost_matrix).unwrap();

    let suggestions = engine.suggest_improvements(&layout);
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
        config: SearchConfig::default(),
        initial_layout: Some(layout),
        pinned_keys: vec![],
        cost_matrix: vec![],
    };

    let suggestions = suggest_improvements(&req).unwrap();
    // Just verify it runs without panic
    assert!(suggestions.len() <= 5);
}

#[test]
fn test_swap_degradation() {
    let kb = setup_kb();
    let mut corpus = Corpus::default();
    corpus.bigrams.push((0, 1, 1000));
    
    let mut rubric = Rubric::default();
    rubric.travel_lat = 10.0;
    
    let cost_matrix = vec![];
    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &cost_matrix).unwrap();
    
    // Optimal Layout: 0, 1, 2. (0 and 1 are adjacent)
    let layout_keys = vec![KeyCode(0), KeyCode(1), KeyCode(2)];
    let mut pos_map = vec![65535u16; 65536];
    pos_map[0] = 0; pos_map[1] = 1; pos_map[2] = 2;
    
    // Propose swapping 1 and 2.
    // 1 moves from x=1 to x=2. Distance from 0 (x=0) increases from 1 to 2.
    // Cost should INCREASE. Delta should be POSITIVE.
    let delta = engine.calculate_swap_delta(&layout_keys, &pos_map, 1, 2).unwrap();
    
    assert!(delta > 0, "Degrading swap should have positive delta");
}