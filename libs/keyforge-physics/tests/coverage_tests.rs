use keyforge_model::{Corpus, KeyNode, Keyboard, Layout, Rubric, SearchConfig, types::{HandIndex, FingerIndex, RowIndex, ColIndex, KeyCode}};
use keyforge_physics::{
    identify, suggest_improvements, verify::DeterministicScorer, EngineRequest, ScoringEngine,
};
use std::sync::Arc;

// --- Helper to create a dummy environment ---
fn setup_env() -> (Arc<Keyboard>, Arc<Corpus>, Arc<Rubric>) {
    let keys: Vec<KeyNode> = (0..30)
        .map(|i| KeyNode {
            index: i,
            label: format!("k{}", i),
            hand: if i < 15 { HandIndex(0) } else { HandIndex(1) },
            finger: FingerIndex((i % 5) as u8),
            row: RowIndex((i / 10) as i8),
            col: ColIndex((i % 10) as i8),
            x: (i % 10) as f32,
            y: (i / 10) as f32,
            is_home: (10..20).contains(&i),
            ..Default::default()
        })
        .collect();

    let kb = Arc::new(Keyboard::new(keys, 1).unwrap());

    let mut corpus = Corpus::default();
    // Add some data to make scoring non-zero
    // Using 'a' (97) and 'b' (98)
    corpus.char_freqs[97] = 1000;
    corpus.char_freqs[98] = 500;
    corpus.bigrams.push((97, 98, 200));

    let rubric = Arc::new(Rubric::default());

    (kb, Arc::new(corpus), rubric)
}

fn setup_simple_kb() -> Keyboard {
    let keys = vec![
        KeyNode {
            index: 0,
            label: "k0".to_string(),
            hand: HandIndex(0),
            finger: FingerIndex(1),
            row: RowIndex(0),
            col: ColIndex(0),
            x: 0.0,
            y: 0.0,
            is_home: true,
            ..Default::default()
        },
        KeyNode {
            index: 1,
            label: "k1".to_string(),
            hand: HandIndex(0),
            finger: FingerIndex(1),
            row: RowIndex(1),
            col: ColIndex(0),
            x: 0.0,
            y: 1.0,
            is_home: false,
            ..Default::default()
        },
        KeyNode {
            index: 2,
            label: "k2".to_string(),
            hand: HandIndex(0),
            finger: FingerIndex(2),
            row: RowIndex(0),
            col: ColIndex(1),
            x: 1.0,
            y: 0.0,
            is_home: true,
            ..Default::default()
        },
    ];
    Keyboard::new(keys, 0).unwrap()
}

#[test]
fn test_fingerprint_identification() {
    // Qwerty standard in fingerprint.rs: "qwertyuiopasdfghjkl;zxcvbnm,./"
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
fn test_heuristics_suggestions() {
    let (kb, corpus, rubric) = setup_env();

    // Create layout with keys present in corpus
    let mut keys: Vec<KeyCode> = (0..30u16).map(KeyCode).collect();
    keys[0] = KeyCode(97);
    keys[1] = KeyCode(98);
    let layout = Layout::new_unchecked(keys);

    let req = EngineRequest {
        keyboard: kb,
        corpus,
        rubric,
        config: SearchConfig::default(),
        initial_layout: Some(layout.clone()),
        pinned_keys: vec![],
        cost_overrides: vec![],
    };

    let suggestions = suggest_improvements(&req).unwrap();
    // We verify it runs without panic and returns a valid vector
    // It might be empty if no swaps improve the score, but it shouldn't crash
    assert!(suggestions.len() <= 5);
}

#[test]
fn test_verify_deterministic_scorer() {
    let (kb, corpus, rubric) = setup_env();

    // Create layout with keys present in corpus
    let mut keys: Vec<KeyCode> = (0..30u16).map(KeyCode).collect();
    keys[0] = KeyCode(97);
    keys[1] = KeyCode(98);
    let layout = Layout::new_unchecked(keys);

    let score = DeterministicScorer::score(&kb, &corpus, &rubric, &layout, &[]);
    assert!(score >= 0.0);

    // Sanity check: Score should be > 0 given our corpus has frequencies
    assert!(score > 0.0, "Score was {}, expected > 0", score);
}

#[test]
fn test_public_api_wrappers() {
    let (kb, corpus, rubric) = setup_env();

    // Create layout with keys present in corpus
    let mut keys: Vec<KeyCode> = (0..30u16).map(KeyCode).collect();
    keys[0] = KeyCode(97);
    keys[1] = KeyCode(98);
    let layout = Layout::new_unchecked(keys);

    let req = EngineRequest {
        keyboard: kb,
        corpus,
        rubric,
        config: SearchConfig::default(),
        initial_layout: Some(layout),
        pinned_keys: vec![],
        cost_overrides: vec![],
    };

    // Test analyze
    let report = keyforge_physics::analyze(&req).unwrap();
    assert!(
        report.score > 0.0,
        "Analyze score was {}, expected > 0",
        report.score
    );

    // Test score
    let opt_res = keyforge_physics::score(&req).unwrap();
    assert!(
        opt_res.score > 0.0,
        "Score result was {}, expected > 0",
        opt_res.score
    );
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
    let layout = Layout::new_unchecked(vec![KeyCode(1), KeyCode(0), KeyCode(2)]);

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
    let layout = Layout::new_unchecked(vec![KeyCode(0), KeyCode(1)]);

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
}
