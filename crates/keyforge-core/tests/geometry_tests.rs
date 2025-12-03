use keyforge_core::config::ScoringWeights;
use keyforge_core::scorer::physics::{analyze_interaction, get_geo_dist};

mod common;
use common::{create_geom, KeyBuilder};

#[test]
fn test_custom_geometry_distance() {
    let k1 = KeyBuilder::new(0, 0).id("k1").pos(0.0, 0.0).build();
    let k2 = KeyBuilder::new(0, 1).id("k2").pos(100.0, 0.0).build();

    let geom = create_geom(vec![k1, k2]);
    let scale = 1.0;

    let dist = get_geo_dist(&geom, 0, 1, scale, scale);
    assert_eq!(dist, 100.0);
}

#[test]
fn test_custom_finger_assignment_sfb() {
    let k1 = KeyBuilder::new(0, 0).id("k1").finger(1).build();
    // Force k2 to be same finger (1)
    let k2 = KeyBuilder::new(0, 1).id("k2").finger(1).build();

    let geom = create_geom(vec![k1, k2]);
    let weights = ScoringWeights::default();

    let result = analyze_interaction(&geom, 0, 1, &weights);
    assert!(result.is_sfb);
}

#[test]
fn test_custom_hand_assignment() {
    let k1 = KeyBuilder::new(0, 0).hand(0).build();
    let k2 = KeyBuilder::new(0, 1).hand(1).build();

    let geom = create_geom(vec![k1, k2]);
    let weights = ScoringWeights::default();
    let scale = 1.0;

    let dist = get_geo_dist(&geom, 0, 1, scale, scale);
    assert_eq!(dist, 0.0);

    let analysis = analyze_interaction(&geom, 0, 1, &weights);
    assert!(!analysis.is_same_hand);
}

#[test]
fn test_non_sfb_lateral_stretch() {
    // Index stretch
    let k1 = KeyBuilder::new(0, 4)
        .id("k1")
        .finger(1)
        .pos(4.0, 0.0)
        .stretch(true)
        .build();
    // Middle finger
    let k2 = KeyBuilder::new(0, 3)
        .id("k2")
        .finger(2)
        .pos(3.0, 0.0)
        .build();

    let geom = create_geom(vec![k1, k2]);
    let weights = ScoringWeights::default();

    let result = analyze_interaction(&geom, 0, 1, &weights);
    assert!(!result.is_sfb);
    assert!(result.is_lateral_stretch);
}
