use keyforge_model::{Corpus, KeyNode, Keyboard, Layout, Rubric};
use keyforge_physics::verify::DeterministicScorer;

fn setup_kb() -> Keyboard {
    // 5 keys in a row
    let keys: Vec<KeyNode> = (0..5)
        .map(|i| KeyNode {
            id: i,
            label: format!("k{}", i),
            hand: 0,
            finger: i as u8,
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
fn test_verify_nan_handling() {
    let kb = setup_kb();
    let layout = Layout::new(vec![0, 1, 2, 3, 4]);
    let mut corpus = Corpus::default();
    corpus.bigrams.push((0, 1, 100));

    let rubric = Rubric {
        travel_lat: f32::NAN, // Should return 0 cost for distance
        ..Rubric::default()
    };

    let score = DeterministicScorer::score(&kb, &corpus, &rubric, &layout);
    // distance cost should be 0 because weight is NaN -> 0
    // but there might be other costs like SFB (finger 0->1 is not SFB)
    assert!(score >= 0.0);
    assert!(!score.is_nan());
}

#[test]
fn test_verify_saturation_max() {
    let kb = setup_kb();
    let layout = Layout::new(vec![0, 1, 2, 3, 4]);
    let mut corpus = Corpus::default();
    corpus.bigrams.push((0, 1, 1));

    let rubric = Rubric {
        travel_lat: 1e30, // Huge weight to force saturation
        ..Rubric::default()
    };

    let score = DeterministicScorer::score(&kb, &corpus, &rubric, &layout);
    assert!(score > 1_000_000.0);
    assert!(score.is_finite());
}

#[test]
fn test_verify_saturation_min() {
    let kb = setup_kb();
    let layout = Layout::new(vec![0, 1, 2, 3, 4]);
    let mut corpus = Corpus::default();
    corpus.bigrams.push((0, 1, 1));

    let rubric = Rubric {
        travel_lat: -1e30, // Huge negative weight
        ..Rubric::default()
    };

    let score = DeterministicScorer::score(&kb, &corpus, &rubric, &layout);
    assert!(score < -1_000_000.0);
    assert!(score.is_finite());
}

#[test]
fn test_verify_trigram_redirects() {
    // Tests: return rubric.redirect; inside calculate_flow_cost_int
    let kb = setup_kb();
    let layout = Layout::new(vec![0, 1, 2, 3, 4]);
    
    // Redirect: 0 -> 1 -> 0 (Finger 0 -> 1 -> 0)
    let mut corpus = Corpus::default();
    corpus.trigrams.push((0, 1, 0, 100)); 

    let rubric = Rubric {
        redirect: 10.0,
        ..Rubric::default()
    };

    let score = DeterministicScorer::score(&kb, &corpus, &rubric, &layout);
    assert!(score > 0.0);
}

#[test]
fn test_verify_trigram_rolls_negative_dir() {
    // Tests: if dir1 < 0 { return rubric.roll_bonus.saturating_neg(); }
    let kb = setup_kb();
    let layout = Layout::new(vec![0, 1, 2, 3, 4]);
    
    // Roll In: 2 -> 1 -> 0 (Finger 2 -> 1 -> 0)
    // dir1 = 1-2 = -1 (neg)
    // dir2 = 0-1 = -1 (neg)
    // Same sign, dir1 < 0 -> Roll Bonus
    let mut corpus = Corpus::default();
    corpus.trigrams.push((2, 1, 0, 100));

    let rubric = Rubric {
        roll_bonus: 5.0,
        ..Rubric::default()
    };

    let score = DeterministicScorer::score(&kb, &corpus, &rubric, &layout);
    assert!(score < 0.0); // Should be negative because roll_bonus reduces cost
}

#[test]
fn test_verify_trigram_zero_dir() {
    // Tests: if dir1 == 0 || dir2 == 0 { return 0; }
    let kb = setup_kb();
    let layout = Layout::new(vec![0, 1, 2, 3, 4]);
    
    // 0 -> 0 -> 1 (Finger 0 -> 0 -> 1)
    // dir1 = 0
    let mut corpus = Corpus::default();
    corpus.trigrams.push((0, 0, 1, 100));

    let _score = DeterministicScorer::score(&kb, &corpus, &Rubric::default(), &layout);
    // Should be 0 flow cost
    // Distance/SFB costs from bigrams are separate, DeterministicScorer sums them all.
    // We can isolate by making other weights 0
    let rubric = Rubric {
        travel_lat: 0.0, travel_vert: 0.0, sfb_base: 0.0, sfb_lateral: 0.0,
        redirect: 10.0, roll_bonus: 10.0,
        ..Rubric::default()
    };
    let score = DeterministicScorer::score(&kb, &corpus, &rubric, &layout);
    assert_eq!(score, 0.0);
}

#[test]
fn test_verify_trigram_redirect_signs() {
    // Tests: if dir1.signum() != dir2.signum() { return rubric.redirect; }
    let kb = setup_kb();
    let layout = Layout::new(vec![0, 1, 2, 3, 4]);
    
    // 0 -> 1 -> 0 (Finger 0 -> 1 -> 0) handled by f1==f3 check?
    // Let's try 0 -> 2 -> 1 (Finger 0 -> 2 -> 1)
    // dir1 = 2-0 = 2 (+)
    // dir2 = 1-2 = -1 (-)
    // Signs differ -> Redirect
    let mut corpus = Corpus::default();
    corpus.trigrams.push((0, 2, 1, 100));

    let rubric = Rubric {
        redirect: 10.0,
        travel_lat: 0.0, travel_vert: 0.0, sfb_base: 0.0, sfb_lateral: 0.0,
        ..Rubric::default()
    };
    
    let score = DeterministicScorer::score(&kb, &corpus, &rubric, &layout);
    assert!(score > 0.0);
}

#[test]
fn test_verify_extreme_coordinates() {
    // Huge coordinates to force overflow of internal i128 distance calc into saturation
    // dx = 30000. dx^2 = 900,000,000.
    // Scale^2 = 1,000,000.
    // Term ~ 900 * Rubric(Max).
    let mut keys = Vec::new();
    keys.push(KeyNode {
         id: 0, label: "k1".to_string(), hand: 0, finger: 1, row: 0, col: 0, x: 0.0, y: 0.0, is_home: false
    });
    keys.push(KeyNode {
         id: 1, label: "k2".to_string(), hand: 0, finger: 2, row: 0, col: 1, x: 30000.0, y: 0.0, is_home: false
    });
    let kb = Keyboard::new(keys, 0);

    let mut corpus = Corpus::default();
    corpus.bigrams.push((0, 1, 100));

    // Rubric with max weights
    let mut rubric = Rubric::default();
    rubric.travel_lat = 1_000_000_000_000.0; // Very large but not infinite

    let layout = Layout::new(vec![0, 1]); 

    let score = DeterministicScorer::score(&kb, &corpus, &rubric, &layout);

    // Should be valid and non-zero
    assert!(score > 0.0);
}
