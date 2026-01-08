// libs/keyforge-physics/tests/robustness.rs

//! Numerical stress tests and robustness checks for the physics engine. Verifies the
//! engine's stability against edge-case inputs—including infinite or NaN rubric weights,
//! frequency saturation, and out-of-bounds swap indices—and ensures the compiler correctly
//! enforces trigram pruning limits and coordinate fallbacks.


use keyforge_model::{
    Corpus, KeyNode, Keyboard, Layout, Rubric, 
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

    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &[]).unwrap();
    let score = engine.score(&layout).unwrap();

    // Should be clamped to MAX but finite
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

    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &[]).unwrap();
    let score = engine.score(&layout).unwrap();

    assert!(score >= 0.0);
    assert!(!score.is_nan());
}

#[test]
fn test_saturation_protection() {
    let kb = setup_kb();
    let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98), KeyCode(99), KeyCode(100), KeyCode(101)]);
    let mut corpus = Corpus::default();
    corpus.bigrams.push((97, 98, u32::MAX)); // Massive frequency

    let rubric = Rubric {
        travel_lat: 1_000_000.0,
        ..Rubric::default()
    };

    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &[]).unwrap();
    let score = engine.score(&layout).unwrap();
    assert!(score.is_finite());
}

#[test]
fn test_missing_keys_in_layout() {
    let kb = setup_kb();
    // Layout missing key 98 ('b')
    let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(0), KeyCode(99), KeyCode(100), KeyCode(101)]);
    let mut corpus = Corpus::default();
    corpus.bigrams.push((97, 98, 100)); 

    let engine = ScoringEngine::new(&kb, &corpus, &Rubric::default(), &[]).unwrap();
    let score = engine.score(&layout).unwrap();
    assert_eq!(score, 0.0); // Should ignore missing pair
}

#[test]
fn test_swap_delta_bounds() {
    let kb = setup_kb();
    let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98), KeyCode(99), KeyCode(100), KeyCode(101)]);
    let mut corpus = Corpus::default();
    corpus.bigrams.push((97, 98, 100));

    let engine = ScoringEngine::new(&kb, &corpus, &Rubric::default(), &[]).unwrap();
    let mut pos_map = vec![65535u16; 65536];
    for (i, &code) in layout.keys.iter().enumerate() {
        pos_map[code.0 as usize] = i as u16;
    }

    // Test out of bounds indices
    let delta = engine.calculate_swap_delta(&layout.keys, &pos_map, 0, 100).unwrap();
    assert_eq!(delta, 0);
}

#[test]
fn test_analyze_layout_empty() {
    let kb = setup_kb();
    let corpus = Corpus::default();
    let rubric = Rubric::default();
    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &[]).unwrap();
    
    // Layout size 0 vs key count 5 -> Should return Error, not panic
    let layout = Layout::new_unchecked(vec![]);
    let result = engine.analyze(&layout);
    assert!(result.is_err());
}

#[test]
fn test_compiler_trigram_pruning() {
    let kb = setup_kb();
    let mut corpus = Corpus::default();
    // Add 20 trigrams
    for i in 0..20 {
        corpus.trigrams.push((0, 1, i as u16, 100));
    }
    
    let mut rubric = Rubric::default();
    rubric.trigram_limit = 5; // Strict limit
    
    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &[]).unwrap();
    
    // Verify using public accessor
    assert_eq!(engine.trigram_count(), 5);
}

#[test]
fn test_compiler_cost_overrides() {
    let kb = setup_kb();
    let mut corpus = Corpus::default();
    corpus.bigrams.push((0, 1, 1));
    
    let rubric = Rubric::default();
    
    // Override cost between Key 0 and Key 1 to be massive (1000.0)
    let overrides = vec![(0, 1, 1000.0)];
    
    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &overrides).unwrap();
    let layout = Layout::new_unchecked(vec![KeyCode(0), KeyCode(1), KeyCode(2), KeyCode(3), KeyCode(4)]);
    
    let score = engine.score(&layout).unwrap();
    assert!(score >= 1000.0);
}

#[test]
fn test_finger_origin_fallback() {
    // Create a keyboard where Finger 1 has keys, but NONE are is_home=true.
    // Compiler should not panic.
    let keys = vec![
        KeyNode { index: 0, finger: FingerIndex(1), is_home: false, ..Default::default() }
    ];
    let kb = Keyboard::new(keys, 0).unwrap();
    let corpus = Corpus::default();
    let rubric = Rubric::default();
    
    let result = ScoringEngine::new(&kb, &corpus, &rubric, &[]);
    assert!(result.is_ok());
}