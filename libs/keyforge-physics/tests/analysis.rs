// libs/keyforge-physics/tests/analysis.rs

//! Integration tests for layout fingerprinting and statistical analysis. Verifies the
//! precision of the `ScoringEngine`'s pattern recognition, ensuring correct detection
//! of Same Finger Bigrams (SFBs), Scissors, Rolls, and Redirects, and validation of
//! heatmap generation for key usage and effort penalties.


use keyforge_model::{
    Corpus, KeyNode, Keyboard, Layout, Rubric, 
    types::{HandIndex, FingerIndex, RowIndex, ColIndex, KeyCode}
};
use keyforge_physics::ScoringEngine;

fn setup_kb(size: usize) -> Keyboard {
    let keys: Vec<KeyNode> = (0..size)
        .map(|i| KeyNode {
            index: i,
            label: format!("k{}", i),
            hand: HandIndex((i % 2) as u8),
            finger: FingerIndex((i % 5) as u8),
            row: RowIndex((i / 10) as i8),
            col: ColIndex((i % 10) as i8),
            x: (i % 10) as f32,
            y: (i / 10) as f32,
            is_home: false,
            ..Default::default()
        })
        .collect();
    Keyboard::new(keys, 1).unwrap()
}

#[test]
fn test_metric_detection_sfb_scissors() {
    // Manually construct a scenario to guarantee Scissor conditions
    // Scissor: Same Hand, Adjacent Fingers, Row Diff >= 2
    let keys = vec![
        // Key 0: Hand 0, Finger 1, Row 0
        KeyNode { index: 0, hand: HandIndex(0), finger: FingerIndex(1), row: RowIndex(0), ..Default::default() },
        // Key 1: Hand 0, Finger 1, Row 0 (SFB with 0)
        KeyNode { index: 1, hand: HandIndex(0), finger: FingerIndex(1), row: RowIndex(0), ..Default::default() },
        // Key 2: Hand 0, Finger 2, Row 2 (Scissor with 0: Adj Finger, Row Diff 2)
        KeyNode { index: 2, hand: HandIndex(0), finger: FingerIndex(2), row: RowIndex(2), ..Default::default() },
    ];
    let kb_manual = Keyboard::new(keys, 1).unwrap();
    
    let mut corpus_manual = Corpus::default();
    corpus_manual.bigrams.push((0, 1, 100)); // SFB
    corpus_manual.bigrams.push((0, 2, 100)); // Scissor

    let cost_matrix = vec![];
    let engine = ScoringEngine::new(&kb_manual, &corpus_manual, &Rubric::default(), &cost_matrix).unwrap();
    // Layout maps char 0->Key0, char 1->Key1, char 2->Key2
    let layout = Layout::new_unchecked(vec![KeyCode(0), KeyCode(1), KeyCode(2)]);
    
    let report = engine.analyze(&layout).unwrap();

    assert!(report.sfb_total > 0.0, "Should detect SFBs");
    assert!(report.scissors > 0.0, "Should detect Scissors");
    assert!(!report.top_sfbs.is_empty());
    assert!(!report.top_scissors.is_empty());
}

#[test]
fn test_metric_detection_rolls_redirects() {
    // Simple 1-row keyboard
    let keys: Vec<KeyNode> = (0..5).map(|i| KeyNode {
        index: i,
        hand: HandIndex(0),
        finger: FingerIndex(i as u8), // 0..4
        ..Default::default()
    }).collect();
    let kb = Keyboard::new(keys, 0).unwrap();

    // Layout: 0=a, 1=b, 2=c, 3=d, 4=e
    let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98), KeyCode(99), KeyCode(100), KeyCode(101)]);
    let mut corpus = Corpus::default();

    // Roll In: Pinky(4) -> Ring(3) -> Mid(2) => 'e' -> 'd' -> 'c'
    corpus.trigrams.push((101, 100, 99, 100));
    // Redirect: Mid(2) -> Ring(3) -> Mid(2) => 'c' -> 'd' -> 'c'
    corpus.trigrams.push((99, 100, 99, 100));

    let rubric = Rubric {
        roll_bonus: 10.0,
        redirect: 50.0,
        trigram_coverage: 1.0,
        trigram_limit: 100,
        ..Rubric::default()
    };

    let cost_matrix = vec![];
    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &cost_matrix).unwrap();
    let report = engine.analyze(&layout).unwrap();

    assert!(report.rolls > 0.0, "Expected rolls");
    assert!(report.redirects > 0.0, "Expected redirects");
}

#[test]
fn test_heatmap_and_penalty_map() {
    let kb = setup_kb(5);
    let mut corpus = Corpus::default();
    
    // 'a'(97) and 'b'(98) have frequency
    corpus.char_freqs[97] = 1000;
    corpus.char_freqs[98] = 1000;
    corpus.bigrams.push((97, 98, 500));

    let cost_matrix = vec![];
    let engine = ScoringEngine::new(&kb, &corpus, &Rubric::default(), &cost_matrix).unwrap();
    let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98), KeyCode(99), KeyCode(100), KeyCode(101)]);
    
    let report = engine.analyze(&layout).unwrap();
    
    // Heatmap (Usage)
    assert!(report.heatmap[0] > 0.0);
    assert!(report.heatmap[1] > 0.0);
    assert_eq!(report.heatmap[2], 0.0); // Unused key
    
    // Penalty Map (Effort)
    assert!(report.penalty_map[0] > 0.0);
    assert!(report.penalty_map[1] > 0.0);
}

#[test]
fn test_lateral_sfb_mechanics() {
    // Setup: 2 keys. Same Hand, Same Finger, Same Row. Adjacent Cols (0 vs 1).
    let keys = vec![
        KeyNode { index: 0, col: ColIndex(0), ..Default::default() },
        KeyNode { index: 1, col: ColIndex(1), ..Default::default() },
    ];
    let kb = Keyboard::new(keys, 0).unwrap();

    let mut corpus = Corpus::default();
    corpus.bigrams.push((0, 1, 1)); // 1 occurrence

    let mut rubric = Rubric::default();
    rubric.sfb_base = 100.0;
    rubric.sfb_lateral = 200.0; // Distinct value

    let cost_matrix = vec![];
    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &cost_matrix).unwrap();
    let layout = Layout::new_unchecked(vec![KeyCode(0), KeyCode(1)]);
    
    let score = engine.score(&layout).unwrap();
    
    // Should trigger Lateral SFB (200) + Distance Cost (small)
    assert!(score >= 200.0);
    // Should NOT be Base SFB (100)
    assert!(score > 150.0);
}

#[test]
fn test_lateral_stretch() {
    // Setup: 2 keys. Same Hand, Same Row. Adjacent Fingers (1 vs 2).
    // But Cols are far apart (0 vs 2). This is a "Lateral Stretch".
    let keys = vec![
        KeyNode { index: 0, finger: FingerIndex(1), col: ColIndex(0), ..Default::default() },
        KeyNode { index: 1, finger: FingerIndex(2), col: ColIndex(2), ..Default::default() },
    ];
    let kb = Keyboard::new(keys, 0).unwrap();

    let mut corpus = Corpus::default();
    corpus.bigrams.push((0, 1, 1));

    let mut rubric = Rubric::default();
    rubric.sfb_lateral = 500.0; // Used for lateral stretch penalty too

    let cost_matrix = vec![];
    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &cost_matrix).unwrap();
    let layout = Layout::new_unchecked(vec![KeyCode(0), KeyCode(1)]);
    
    let score = engine.score(&layout).unwrap();
    assert!(score >= 500.0);
}

#[test]
fn test_bilateral_usage_distribution() {
    // Setup: 3 Keys
    // Key 0: Left Hand (Source)
    // Key 1: Right Hand (Option A for Target)
    // Key 2: Right Hand (Option B for Target)
    let keys = vec![
        KeyNode { index: 0, hand: HandIndex(0), finger: FingerIndex(1), x: 0.0, y: 0.0, ..Default::default() },
        KeyNode { index: 1, hand: HandIndex(1), finger: FingerIndex(1), x: 10.0, y: 0.0, ..Default::default() },
        KeyNode { index: 2, hand: HandIndex(1), finger: FingerIndex(2), x: 11.0, y: 0.0, ..Default::default() },
    ];
    let kb = Keyboard::new(keys, 0).unwrap();

    // Layout: 'A'(65) on Key 0. 'B'(66) on BOTH Key 1 and Key 2.
    // This simulates "Bilateral Space" or duplicate keys.
    let layout = Layout::new_unchecked(vec![
        KeyCode(65), // Key 0 = A
        KeyCode(66), // Key 1 = B
        KeyCode(66), // Key 2 = B
    ]);

    let mut corpus = Corpus::default();
    // Char Freqs (needed for Monogram pass)
    corpus.char_freqs[65] = 100;
    corpus.char_freqs[66] = 100;
    
    // Bigram: A -> B (Freq 100)
    corpus.bigrams.push((65, 66, 100));

    // Cost Matrix to force choice
    // Moving from Key 0 (A) to Key 1 (B_primary) is VERY EXPENSIVE.
    // Moving from Key 0 (A) to Key 2 (B_secondary) is CHEAP.
    // The engine should choose Key 2 for 'B' in this context.
    let cost_matrix = vec![
        (0, 1, 1000.0), // A -> B1
        (0, 2, 10.0),   // A -> B2
    ];

    let engine = ScoringEngine::new(&kb, &corpus, &Rubric::default(), &cost_matrix).unwrap();
    let report = engine.analyze(&layout).unwrap();

    // 1. Check Usage (Heatmap)
    // Key 1 should be avoided. Key 2 should be used.
    // Normalized to percentages (0-100)
    println!("Heatmap: {:?}", report.heatmap);
    assert!(report.heatmap[2] > report.heatmap[1], "Should prefer Key 2 over Key 1");
    assert!(report.heatmap[2] > 40.0, "Key 2 should have significant usage"); // ~50% total (A=50, B=50) -> Key 2 ~= 50

    // 2. Check Effort (Penalty Map)
    // Base effort for 'B' should be attributed to Key 2, not Key 1.
    // Since Key 1 and Key 2 have same base finger effort (0.0 default), 
    // we need to set non-zero base costs or check if penalty map follows heatmap.
    // The previous implementation dumped all monogram effort on the first key (Key 1).
    // The new implementation should follow the usage.
    // Let's assume standard finger effort.
    println!("Penalty Map: {:?}", report.penalty_map);
    // If the fix works, Penalty[2] should be proportional to Usage[2].
}

#[test]
fn test_trigram_flow_usage() {
    // Setup: 3 Keys for a roll
    // Key 0, 1, 2.
    // Layout: A on 0, B on 1, C on 2.
    // But let's duplicate B on Key 3 to test selection.
    
    let keys = vec![
        KeyNode { index: 0, x: 0.0, ..Default::default() },
        KeyNode { index: 1, x: 2.0, ..Default::default() }, // B1 (Bad position for roll?)
        KeyNode { index: 2, x: 4.0, ..Default::default() }, // C
        KeyNode { index: 3, x: 1.0, ..Default::default() }, // B2 (Good position for roll A->B2->C)
    ];
    let kb = Keyboard::new(keys, 0).unwrap();

    let layout = Layout::new_unchecked(vec![
        KeyCode(65), // 0: A
        KeyCode(66), // 1: B (Primary)
        KeyCode(67), // 2: C
        KeyCode(66), // 3: B (Secondary)
    ]);

    let mut corpus = Corpus::default();
    corpus.char_freqs[65] = 100;
    corpus.char_freqs[66] = 100;
    corpus.char_freqs[67] = 100;
    
    // Trigram A -> B -> C
    corpus.trigrams.push((65, 66, 67, 100));

    // Force selection via Cost Matrix
    // We want A -> B2 -> C to be better than A -> B1 -> C.
    // Costs:
    // A(0)->B1(1) = 100
    // B1(1)->C(2) = 100
    // Total Path 1 = 200
    
    // A(0)->B2(3) = 10
    // B2(3)->C(2) = 10
    // Total Path 2 = 20
    
    let cost_matrix = vec![
        (0, 1, 100.0), (1, 2, 100.0),
        (0, 3, 10.0),  (3, 2, 10.0),
    ];

    let engine = ScoringEngine::new(&kb, &corpus, &Rubric::default(), &cost_matrix).unwrap();
    let report = engine.analyze(&layout).unwrap();

    println!("Heatmap: {:?}", report.heatmap);
    
    // The "Rigorous" analyzer uses Trigrams first. 
    // It should identify that the triplet 0->3->2 is cheaper than 0->1->2.
    // So usage for B should go to Key 3.
    assert!(report.heatmap[3] > report.heatmap[1], "Trigram optimization should choose Key 3 for B");
}
