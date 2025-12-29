use keyforge_model::{Corpus, KeyNode, Keyboard, Layout, Rubric};
use keyforge_physics::ScoringEngine;

fn setup_simple_kb() -> Keyboard {
    let keys = vec![
        KeyNode {
            id: 0,
            label: "k0".to_string(),
            hand: 0,
            finger: 1,
            row: 0,
            col: 0,
            x: 0.0,
            y: 0.0,
            is_home: true,
        },
        KeyNode {
            id: 1,
            label: "k1".to_string(),
            hand: 0,
            finger: 1,
            row: 1,
            col: 0,
            x: 0.0,
            y: 1.0,
            is_home: false,
        },
        KeyNode {
            id: 2,
            label: "k2".to_string(),
            hand: 0,
            finger: 2,
            row: 0,
            col: 1,
            x: 1.0,
            y: 0.0,
            is_home: true,
        },
    ];
    Keyboard::new(keys, 0)
}

#[test]
fn test_heuristics_swap_suggestion_success() {
    let kb = setup_simple_kb();
    let mut corpus = Corpus::default();

    // Bigram (0, 2) -> High Freq (1000). Char 0 wants to be close to Char 2.
    // Bigram (1, 2) -> Low Freq (1).
    corpus.bigrams.push((0, 2, 1000));
    corpus.bigrams.push((1, 2, 1));

    // Char 2 is on Key 2 (Fixed anchor).
    // Char 0 is on Key 1 (Far from K2).
    // Char 1 is on Key 0 (Close to K2).

    // Rubric
    let mut rubric = Rubric::default();
    rubric.travel_vert = 10.0;
    rubric.travel_lat = 10.0;

    // Layout: Char 1 on K0, Char 0 on K1, Char 2 on K2.
    // Code 1 -> Pos 0 (K0). Code 0 -> Pos 1 (K1). Code 2 -> Pos 2 (K2).
    let layout = Layout::new_unchecked(vec![1, 0, 2]);

    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &[]).unwrap();

    // Score Analysis:
    // (0, 2): K1 to K2. K1(0,1), K2(1,0). dx=1, dy=1. cost=20. Total=20000.
    // (1, 2): K0 to K2. K0(0,0), K2(1,0). dx=1, dy=0. cost=10. Total=10.
    // Total = 20010.

    // Swap 0 and 1:
    // Char 0 on K0. (0, 2): K0 to K2. Cost=10. Total=10000.
    // Char 1 on K1. (1, 2): K1 to K2. Cost=20. Total=20.
    // New Total = 10020.
    // Improvement ~ 50%.

    let suggestions = engine.suggest_improvements(&layout);

    assert!(!suggestions.is_empty(), "Should suggest a swap");
    let s = &suggestions[0];
    assert!(s.improvement_pct > 10.0);
}

#[test]
fn test_heuristics_zero_score_early_return() {
    let kb = setup_simple_kb();
    let corpus = Corpus::default(); // Empty corpus = 0 score
    let rubric = Rubric::default();
    let layout = Layout::new_unchecked(vec![0, 1]);

    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &[]).unwrap();

    let suggestions = engine.suggest_improvements(&layout);
    assert!(
        suggestions.is_empty(),
        "Zero score should return empty suggestions"
    );
}

#[test]
fn test_lib_accessors() {
    let kb = setup_simple_kb();
    let corpus = Corpus::default();
    let rubric = Rubric::default();
    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &[]).unwrap();

    assert_eq!(engine.key_count(), 3);
    // Removed private field access check
}
