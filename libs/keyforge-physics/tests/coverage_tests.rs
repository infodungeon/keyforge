use keyforge_model::{Corpus, KeyNode, Keyboard, Layout, Rubric, SearchConfig};
use keyforge_physics::{
    identify, suggest_improvements, verify::DeterministicScorer, EngineRequest,
};
use std::sync::Arc;

// --- Helper to create a dummy environment ---
fn setup_env() -> (Arc<Keyboard>, Arc<Corpus>, Arc<Rubric>) {
    let keys: Vec<KeyNode> = (0..30)
        .map(|i| KeyNode {
            id: i,
            label: format!("k{}", i),
            hand: if i < 15 { 0 } else { 1 },
            finger: (i % 5) as u8,
            row: (i / 10) as i8,
            col: (i % 10) as i8,
            x: (i % 10) as f32,
            y: (i / 10) as f32,
            is_home: (10..20).contains(&i),
        })
        .collect();

    let kb = Arc::new(Keyboard::new(keys, 1));

    let mut corpus = Corpus::default();
    // Add some data to make scoring non-zero
    // Using 'a' (97) and 'b' (98)
    corpus.char_freqs[97] = 1000;
    corpus.char_freqs[98] = 500;
    corpus.bigrams.push((97, 98, 200));

    let rubric = Arc::new(Rubric::default());

    (kb, Arc::new(corpus), rubric)
}

#[test]
fn test_fingerprint_identification() {
    // Qwerty standard in fingerprint.rs: "qwertyuiopasdfghjkl;zxcvbnm,./"
    let qwerty_str = "qwertyuiopasdfghjkl;zxcvbnm,./";
    let keys: Vec<u16> = qwerty_str.chars().map(|c| c as u16).collect();
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
    let mut keys: Vec<u16> = (0..30).map(|i| i as u16).collect();
    keys[0] = 97;
    keys[1] = 98;
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

    let suggestions = suggest_improvements(&req);
    // We verify it runs without panic and returns a valid vector
    // It might be empty if no swaps improve the score, but it shouldn't crash
    assert!(suggestions.len() <= 5);
}

#[test]
fn test_verify_deterministic_scorer() {
    let (kb, corpus, rubric) = setup_env();

    // Create layout with keys present in corpus
    let mut keys: Vec<u16> = (0..30).map(|i| i as u16).collect();
    keys[0] = 97;
    keys[1] = 98;
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
    let mut keys: Vec<u16> = (0..30).map(|i| i as u16).collect();
    keys[0] = 97;
    keys[1] = 98;
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
    let report = keyforge_physics::analyze(&req);
    assert!(
        report.score > 0.0,
        "Analyze score was {}, expected > 0",
        report.score
    );

    // Test score
    let opt_res = keyforge_physics::score(&req);
    assert!(
        opt_res.score > 0.0,
        "Score result was {}, expected > 0",
        opt_res.score
    );
}
