// ===== keyforge/tests/geometry_tests.rs =====
use keyforge::config::ScoringWeights;
use keyforge::geometry::{KeyNode, KeyboardGeometry};
use keyforge::scorer::physics::{analyze_interaction, get_geo_dist};

fn make_2key_geom(k1: KeyNode, k2: KeyNode) -> KeyboardGeometry {
    let mut geom = KeyboardGeometry {
        keys: vec![k1, k2],
        prime_slots: vec![],
        med_slots: vec![],
        low_slots: vec![],
        home_row: 0,
        finger_origins: [[(0.0, 0.0); 5]; 2],
    };
    geom.calculate_origins();
    geom
}

#[test]
fn test_custom_geometry_distance() {
    let k1 = KeyNode {
        id: "k1".to_string(),
        hand: 0,
        finger: 1,
        row: 0,
        col: 0,
        x: 0.0,
        y: 0.0,
        is_stretch: false,
    };
    let k2 = KeyNode {
        id: "k2".to_string(),
        hand: 0,
        finger: 1,
        row: 0,
        col: 1,
        x: 100.0,
        y: 0.0,
        is_stretch: false,
    };

    let geom = make_2key_geom(k1, k2);
    let scale = 1.0;

    let dist = get_geo_dist(&geom, 0, 1, scale, scale);
    assert_eq!(dist, 100.0, "Geometry ignored custom X coordinates");
}

#[test]
fn test_custom_finger_assignment_sfb() {
    let k1 = KeyNode {
        id: "k1".to_string(),
        hand: 0,
        finger: 1,
        row: 0,
        col: 0,
        x: 0.0,
        y: 0.0,
        is_stretch: false,
    };
    let k2 = KeyNode {
        id: "k2".to_string(),
        hand: 0,
        finger: 1,
        row: 0,
        col: 1,
        x: 1.0,
        y: 0.0,
        is_stretch: false,
    };

    let geom = make_2key_geom(k1, k2);
    let weights = ScoringWeights::default();

    let result = analyze_interaction(&geom, 0, 1, &weights);
    assert!(result.is_sfb, "Custom geometry failed to detect SFB");
}

#[test]
fn test_custom_hand_assignment() {
    let k1 = KeyNode {
        id: "k1".to_string(),
        hand: 0,
        finger: 1,
        row: 0,
        col: 0,
        x: 0.0,
        y: 0.0,
        is_stretch: false,
    };
    let k2 = KeyNode {
        id: "k2".to_string(),
        hand: 1,
        finger: 1,
        row: 0,
        col: 1,
        x: 1.0,
        y: 0.0,
        is_stretch: false,
    };

    let geom = make_2key_geom(k1, k2);
    let weights = ScoringWeights::default();
    let scale = 1.0;

    let dist = get_geo_dist(&geom, 0, 1, scale, scale);
    assert_eq!(dist, 0.0, "Cross-hand distance should be 0.0");

    let analysis = analyze_interaction(&geom, 0, 1, &weights);
    assert!(!analysis.is_same_hand, "Hand assignment failed");
}

#[test]
fn test_non_sfb_lateral_stretch() {
    let k1 = KeyNode {
        id: "k1".to_string(),
        hand: 0,
        finger: 1,
        row: 0,
        col: 4,
        x: 4.0,
        y: 0.0,
        is_stretch: true,
    };
    let k2 = KeyNode {
        id: "k2".to_string(),
        hand: 0,
        finger: 2,
        row: 0,
        col: 3,
        x: 3.0,
        y: 0.0,
        is_stretch: false,
    };
    let geom = make_2key_geom(k1, k2);
    let weights = ScoringWeights::default();

    let result = analyze_interaction(&geom, 0, 1, &weights);
    assert!(!result.is_sfb, "Should not be SFB");
    assert!(result.is_lateral_stretch, "Should be Lateral Stretch");
}
