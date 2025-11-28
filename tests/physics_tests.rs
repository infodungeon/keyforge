use keyforge::geometry::KeyboardGeometry;
use keyforge::scorer::physics::{analyze_interaction, get_reach_cost};
use rstest::rstest;

// --- KEY INDEX MAPPING (Standard 30-key Grid) ---
// Row 0 (Top)
const Q: usize = 0; // L Pinky
const W: usize = 1; // L Ring
const E: usize = 2; // L Middle
const R: usize = 3; // L Index
const T: usize = 4; // L Index (Stretch)

// Row 1 (Home)
const A: usize = 10; // L Pinky
const S: usize = 11; // L Ring
const D: usize = 12; // L Middle
const F: usize = 13; // L Index
const G: usize = 14; // L Index (Stretch)
const H: usize = 15; // R Index (Stretch)

// Row 2 (Bottom)
const Z: usize = 20; // L Pinky
const C: usize = 22; // L Middle
const V: usize = 23; // L Index

fn get_geom() -> KeyboardGeometry {
    KeyboardGeometry::standard()
}

// --- SFB TESTS ---
#[rstest]
#[case(Q, A, true)] // Pinky Top -> Pinky Home
#[case(F, R, true)] // Index Home -> Index Top
#[case(F, T, true)] // Index Home -> Index Stretch
#[case(F, V, true)] // Index Home -> Index Bottom
#[case(F, G, true)] // Index Home -> Index Stretch (Same Finger = SFB)
#[case(W, E, false)] // Ring -> Middle (Different fingers)
#[case(A, S, false)] // Pinky -> Ring
fn test_is_sfb(#[case] k1: usize, #[case] k2: usize, #[case] expected: bool) {
    let geom = get_geom();
    let result = analyze_interaction(&geom, k1, k2);
    assert_eq!(
        result.is_sfb, expected,
        "SFB check failed for keys {} -> {}",
        k1, k2
    );
}

// --- LATERAL TESTS (Non-SFB) ---
#[rstest]
// 1. The "Good" Neighbors (No Penalty)
#[case(W, E, false)] // Ring -> Middle
#[case(S, D, false)] // Ring -> Middle
#[case(D, F, false)] // Middle -> Index
// 2. The "Stretch" Neighbors (Lateral Penalty)
#[case(H, G, false)] // Right Index -> Left Index (Cross hand = False)
#[case(A, S, false)] // Pinky -> Ring (Neighbors, no stretch)
fn test_is_lateral_standard(#[case] k1: usize, #[case] k2: usize, #[case] expected: bool) {
    let geom = get_geom();
    let result = analyze_interaction(&geom, k1, k2);
    assert_eq!(
        result.is_lateral_stretch, expected,
        "Lateral check failed for keys {} -> {}",
        k1, k2
    );
}

// --- SCISSOR TESTS ---
#[rstest]
// 1. True Scissors
#[case(R, C, true)] // Top Index (3) -> Bot Middle (22). Row Diff 2. Finger Diff 1.
#[case(W, Z, true)] // Top Ring (1) -> Bot Pinky (20). Row Diff 2. Finger Diff 1.
#[case(E, V, true)] // Top Middle -> Bot Index.
// 2. Not Scissors
#[case(R, D, false)] // Top -> Home (Row Diff 1)
#[case(F, C, false)] // Home -> Bottom (Row Diff 1)
#[case(Q, Z, false)] // Pinky -> Pinky (SFB, not Scissor)
#[case(R, Z, false)] // Index -> Pinky (Finger diff > 1)
fn test_is_scissor(#[case] k1: usize, #[case] k2: usize, #[case] expected: bool) {
    let geom = get_geom();
    let result = analyze_interaction(&geom, k1, k2);
    assert_eq!(
        result.is_scissor, expected,
        "Scissor check failed for keys {} -> {}",
        k1, k2
    );
}

// --- REACH COST TESTS ---
#[rstest]
#[case(A, 0.0)] // Home Row -> 0
#[case(Q, 10.0)] // Top Row -> 1.0 * 10
#[case(Z, 10.0)] // Bot Row -> 1.0 * 10
#[case(G, 10.0)] // Home Stretch -> 1.0 * 10
#[case(T, 14.142)] // Top Stretch -> Sqrt(2) * 10
fn test_reach_costs(#[case] k: usize, #[case] expected: f32) {
    let geom = get_geom();
    let scale = 10.0;
    let cost = get_reach_cost(&geom, k, scale);
    assert!(
        (cost - expected).abs() < 0.01,
        "Reach cost for key {} was {}, expected {}",
        k,
        cost,
        expected
    );
}

// --- ROLL TESTS (NEW) ---
// Definition: Same Hand, Different Finger.
// Inward: High Finger ID -> Low Finger ID (e.g. Pinky 4 -> Index 1)
// Outward: Low Finger ID -> High Finger ID (e.g. Index 1 -> Pinky 4)

#[rstest]
// Inward Rolls
#[case(A, S, true, false)] // Pinky(4) -> Ring(3)
#[case(S, D, true, false)] // Ring(3) -> Middle(2)
#[case(D, F, true, false)] // Middle(2) -> Index(1)
#[case(A, F, true, false)] // Pinky(4) -> Index(1) (Big Inward)

// Outward Rolls
#[case(F, D, false, true)] // Index(1) -> Middle(2)
#[case(D, S, false, true)] // Middle(2) -> Ring(3)
#[case(S, A, false, true)] // Ring(3) -> Pinky(4)
#[case(F, A, false, true)] // Index(1) -> Pinky(4) (Big Outward)

// Not Rolls (SFBs)
#[case(F, R, false, false)] // Index -> Index
#[case(Q, A, false, false)] // Pinky -> Pinky

// Not Rolls (Different Hands)
#[case(F, H, false, false)] // Left Index -> Right Index

fn test_bigram_rolls(
    #[case] k1: usize,
    #[case] k2: usize,
    #[case] expect_in: bool,
    #[case] expect_out: bool,
) {
    let geom = get_geom();
    let result = analyze_interaction(&geom, k1, k2);

    assert_eq!(
        result.is_roll_in, expect_in,
        "Inward roll check failed for {}->{}",
        k1, k2
    );
    assert_eq!(
        result.is_roll_out, expect_out,
        "Outward roll check failed for {}->{}",
        k1, k2
    );
}
