use keyforge_model::{Corpus, KeyNode, Keyboard, Layout, Rubric};
use keyforge_physics::{verify::DeterministicScorer, ScoringEngine};

fn setup_specific_kb() -> Keyboard {
    // Create a simple 1-row keyboard for easy finger logic
    // Hand 0 (Left): Fingers 0, 1, 2, 3, 4
    let keys: Vec<KeyNode> = (0..5)
        .map(|i| KeyNode {
            id: i,
            label: format!("k{}", i),
            hand: 0,
            finger: i as u8, // 0=Thumb, 1=Index, 2=Mid, 3=Ring, 4=Pinky
            row: 0,
            col: i as i8,
            x: i as f32,
            y: 0.0,
            is_home: true,
        })
        .collect();
    Keyboard::new(keys, 0)
}

#[test]
fn test_trigram_rolls_and_redirects() {
    let kb = setup_specific_kb();
    
    // Layout: 0=a, 1=b, 2=c, 3=d, 4=e
    // Keycodes: 97='a', 98='b', 99='c', 100='d', 101='e'
    let layout = Layout::new(vec![97, 98, 99, 100, 101]);

    let mut corpus = Corpus::default();
    
    // Roll In: Pinky(4) -> Ring(3) -> Mid(2) => 'e' -> 'd' -> 'c' (101, 100, 99)
    // Direction: -1, -1 (Consistent inward)
    corpus.trigrams.push((101, 100, 99, 100)); 

    // Redirect: Mid(2) -> Ring(3) -> Mid(2) => 'c' -> 'd' -> 'c' (99, 100, 99)
    // Direction: +1, -1 (Change)
    corpus.trigrams.push((99, 100, 99, 100));

    let rubric = Rubric {
        roll_bonus: 10.0,
        redirect: 50.0,
        trigram_coverage: 1.0,
        trigram_limit: 100,
        ..Rubric::default()
    };

    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &[]);
    let report = engine.analyze(&layout);

    // We expect some rolls and some redirects
    assert!(report.rolls > 0.0, "Expected rolls, got {}", report.rolls);
    assert!(report.redirects > 0.0, "Expected redirects, got {}", report.redirects);
}

#[test]
fn test_math_boundaries_infinity() {
    let kb = setup_specific_kb();
    let layout = Layout::new(vec![97, 98, 99, 100, 101]);
    
    let mut corpus = Corpus::default();
    corpus.bigrams.push((97, 98, 1000)); // a->b

    let rubric = Rubric {
        travel_lat: f32::INFINITY,
        ..Rubric::default()
    };

    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &[]);
    let score = engine.score(&layout);

    // Should be very large but finite (clamped to i64::MAX scaled down)
    assert!(score > 1_000_000.0); 
    assert!(score.is_finite());

    // Verify DeterministicScorer handles it too
    let det_score = DeterministicScorer::score(&kb, &corpus, &rubric, &layout);
    assert!(det_score > 1_000_000.0);
    assert!(det_score.is_finite());
}

#[test]
fn test_math_boundaries_nan() {
    let kb = setup_specific_kb();
    let layout = Layout::new(vec![97, 98, 99, 100, 101]);
    
    let mut corpus = Corpus::default();
    corpus.bigrams.push((97, 98, 1000));

    let rubric = Rubric {
        travel_lat: f32::NAN,
        ..Rubric::default()
    };

    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &[]);
    let score = engine.score(&layout);

    // NaN usually treated as 0 in safe_float_to_int
    assert!(score >= 0.0); 
    // It shouldn't propagate NaN
    assert!(!score.is_nan());
}

#[test]
fn test_saturation_protection() {
    let kb = setup_specific_kb();
    let layout = Layout::new(vec![97, 98, 99, 100, 101]);
    
    let mut corpus = Corpus::default();
    corpus.bigrams.push((97, 98, u32::MAX)); // Massive frequency

    let rubric = Rubric {
        travel_lat: 1_000_000.0,
        ..Rubric::default()
    };

    // This combination would overflow a standard calculation if not saturated
    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &[]);
    let score = engine.score(&layout);
    
    assert!(score.is_finite());
}

#[test]
fn test_missing_keys_in_layout() {
    let kb = setup_specific_kb();
    // Layout missing key 98 ('b')
    let layout = Layout::new(vec![97, 0, 99, 100, 101]); 
    
    let mut corpus = Corpus::default();
    corpus.bigrams.push((97, 98, 100)); // a->b (b is missing)

    let rubric = Rubric::default();
    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &[]);
    
    // Should not panic, score should ignore the missing key pair
    let score = engine.score(&layout);
    assert_eq!(score, 0.0);
}

#[test]
fn test_high_keycodes_safety() {
    let kb = setup_specific_kb();
    // Use keycode 300 (outside optimized 0-255 range)
    let layout = Layout::new(vec![300, 301, 302, 303, 304]);
    
    let mut corpus = Corpus::default();
    corpus.bigrams.push((300, 301, 100));

    let rubric = Rubric::default();
    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &[]);
    
    let score = engine.score(&layout);
    // REMOVED LIMIT: High keycodes are now correctly scored.
    assert!(score > 0.0);
}

#[test]
fn test_swap_delta_bounds() {
    let kb = setup_specific_kb();
    let layout = Layout::new(vec![97, 98, 99, 100, 101]);
    
    let mut corpus = Corpus::default();
    corpus.bigrams.push((97, 98, 100));

    let rubric = Rubric::default();
    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &[]);

    let mut pos_map = vec![65535u16; 65536];
    // Populate pos_map manually to test calculate_swap_delta
    for (i, &code) in layout.keys.iter().enumerate() {
        pos_map[code as usize] = i as u16;
    }

    // Test out of bounds indices
    let delta = engine.calculate_swap_delta(&layout.keys, &pos_map, 0, 100);
    assert_eq!(delta, 0);

    let delta = engine.calculate_swap_delta(&layout.keys, &pos_map, 100, 0);
    assert_eq!(delta, 0);
}

#[test]
fn test_math_boundaries_neg_infinity() {
    let kb = setup_specific_kb();
    let layout = Layout::new(vec![97, 98, 99, 100, 101]);
    
    let mut corpus = Corpus::default();
    corpus.bigrams.push((97, 98, 1000));

    let rubric = Rubric {
        travel_lat: f32::NEG_INFINITY,
        ..Rubric::default()
    };

    let det_score = DeterministicScorer::score(&kb, &corpus, &rubric, &layout);
    // Should saturate to i64::MIN (scaled)
    assert!(det_score < -1_000_000.0);
    assert!(det_score.is_finite());
}
