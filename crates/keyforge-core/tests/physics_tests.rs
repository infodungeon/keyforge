// UPDATED: use keyforge_core
use keyforge_core::config::ScoringWeights;
use keyforge_core::geometry::{KeyNode, KeyboardGeometry};
use keyforge_core::scorer::physics::{analyze_interaction, get_reach_cost};
use rstest::rstest;

// --- KEY INDEX MAPPING (Standard 30-key Grid) ---
// Row 0 (Top)
const Q: usize = 0;
const W: usize = 1;
const E: usize = 2;
const R: usize = 3;
const T: usize = 4;

// Row 1 (Home)
const A: usize = 10;
const S: usize = 11;
const D: usize = 12;
const F: usize = 13;
const G: usize = 14;
const H: usize = 15;

// Row 2 (Bottom)
const Z: usize = 20;
const C: usize = 22;
const V: usize = 23;

// Helper to construct a PERFECT GRID geometry for testing math
fn get_mock_geom() -> KeyboardGeometry {
    let mut keys = Vec::new();
    for r in 0..3 {
        for c in 0..10 {
            let hand = if c < 5 { 0 } else { 1 };
            let finger = match c {
                0 | 9 => 4, // Pinky
                1 | 8 => 3, // Ring
                2 | 7 => 2, // Mid
                3 | 6 => 1, // Index
                4 | 5 => 1, // Index Stretch
                _ => 1,
            };
            let is_stretch = c == 4 || c == 5;

            // Perfect Grid Coordinates
            let x = c as f32;
            let y = r as f32;

            keys.push(KeyNode {
                id: format!("k_{}_{}", r, c),
                hand,
                finger,
                row: r as i8,
                col: c as i8,
                x,
                y,
                is_stretch,
            });
        }
    }

    let mut geom = KeyboardGeometry {
        keys,
        prime_slots: vec![],
        med_slots: vec![],
        low_slots: vec![],
        home_row: 1, // Row 1 is home
        finger_origins: [[(0.0, 0.0); 5]; 2],
    };

    // Manually set the origins for specific testing logic
    // Left Hand (0) Pinky (4): A is at (0, 1)
    geom.finger_origins[0][4] = (0.0, 1.0);
    // Left Hand (0) Index (1): F is at (3, 1)
    geom.finger_origins[0][1] = (3.0, 1.0);

    geom
}

// --- SFB TESTS ---
#[rstest]
#[case(Q, A, true)]
#[case(F, R, true)]
#[case(F, T, true)]
#[case(F, V, true)]
#[case(F, G, true)]
#[case(W, E, false)]
#[case(A, S, false)]
fn test_is_sfb(#[case] k1: usize, #[case] k2: usize, #[case] expected: bool) {
    let geom = get_mock_geom();
    let weights = ScoringWeights::default();
    let result = analyze_interaction(&geom, k1, k2, &weights);
    assert_eq!(result.is_sfb, expected);
}

// --- REACH COST TESTS ---
#[rstest]
#[case(A, 0.0)] // Home (0,1) -> (0,1) = 0
#[case(Q, 10.0)] // Top (0,0) -> (0,1) = dy=1. cost = 1*10 = 10
#[case(Z, 10.0)] // Bot (0,2) -> (0,1) = dy=1. cost = 1*10 = 10
#[case(G, 10.0)] // Stretch (4,1) -> Home (3,1) = dx=1. cost = 1*10 = 10
#[case(T, 14.142)] // Top Stretch (4,0) -> Home (3,1). dx=1, dy=1. sqrt(2)*10
fn test_reach_costs(#[case] k: usize, #[case] expected: f32) {
    let geom = get_mock_geom();
    let scale = 10.0;

    let cost = get_reach_cost(&geom, k, scale, scale);

    assert!(
        (cost - expected).abs() < 0.01,
        "Reach cost for key {} was {}, expected {}",
        k,
        cost,
        expected
    );
}

// --- OTHER TESTS (Lateral, Scissor, Roll) ---

#[rstest]
#[case(W, E, false)]
#[case(S, D, false)]
#[case(D, F, false)]
#[case(H, G, false)]
#[case(A, S, false)]
fn test_is_lateral_standard(#[case] k1: usize, #[case] k2: usize, #[case] expected: bool) {
    let geom = get_mock_geom();
    let weights = ScoringWeights::default();
    let result = analyze_interaction(&geom, k1, k2, &weights);
    assert_eq!(result.is_lateral_stretch, expected);
}

#[rstest]
#[case(R, C, true)] // Index Top / Middle Bot -> Scissor
#[case(W, Z, false)] // Ring Top / Pinky Bot -> Comfortable in default weights (34)
#[case(E, V, false)] // Middle Top / Index Bot -> Comfortable in default weights (21)
#[case(R, D, false)] // Not row diff >= 2
#[case(F, C, false)] // Same finger SFB
#[case(Q, Z, false)] // Not adjacent fingers
#[case(R, Z, false)] // Not adjacent fingers
fn test_is_scissor(#[case] k1: usize, #[case] k2: usize, #[case] expected: bool) {
    let geom = get_mock_geom();
    let weights = ScoringWeights::default(); // Uses default comfortable_scissors: "21,23,34"
    let result = analyze_interaction(&geom, k1, k2, &weights);
    assert_eq!(result.is_scissor, expected);
}

#[rstest]
#[case(A, S, true, false)]
#[case(S, D, true, false)]
#[case(D, F, true, false)]
#[case(A, F, true, false)]
#[case(F, D, false, true)]
#[case(D, S, false, true)]
#[case(S, A, false, true)]
#[case(F, A, false, true)]
#[case(F, R, false, false)]
#[case(Q, A, false, false)]
#[case(F, H, false, false)]
fn test_bigram_rolls(
    #[case] k1: usize,
    #[case] k2: usize,
    #[case] expect_in: bool,
    #[case] expect_out: bool,
) {
    let geom = get_mock_geom();
    let weights = ScoringWeights::default();
    let result = analyze_interaction(&geom, k1, k2, &weights);
    assert_eq!(result.is_roll_in, expect_in);
    assert_eq!(result.is_roll_out, expect_out);
}
