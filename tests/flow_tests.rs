use keyforge::config::ScoringWeights;
use keyforge::geometry::{KeyNode, KeyboardGeometry};
use keyforge::scorer::flow::analyze_flow;
use keyforge::scorer::physics::analyze_interaction;

// Mock 3-key Geometry (Same Hand)
// 0: Index Home (1)
// 1: Middle Home (2)
// 2: Ring Home (3)
fn get_roll_geom() -> KeyboardGeometry {
    let keys = vec![
        // Index (Finger 1)
        KeyNode {
            id: "idx".to_string(),
            hand: 0,
            finger: 1,
            row: 1,
            col: 3,
            x: 3.0,
            y: 1.0,
            is_stretch: false,
        },
        // Middle (Finger 2)
        KeyNode {
            id: "mid".to_string(),
            hand: 0,
            finger: 2,
            row: 1,
            col: 2,
            x: 2.0,
            y: 1.0,
            is_stretch: false,
        },
        // Ring (Finger 3)
        KeyNode {
            id: "rng".to_string(),
            hand: 0,
            finger: 3,
            row: 1,
            col: 1,
            x: 1.0,
            y: 1.0,
            is_stretch: false,
        },
    ];

    let mut geom = KeyboardGeometry {
        keys,
        prime_slots: vec![],
        med_slots: vec![],
        low_slots: vec![],
        home_row: 1,
        finger_origins: [[(0.0, 0.0); 5]; 2],
    };
    geom.calculate_origins();
    geom
}

#[test]
fn test_bigram_roll_detection() {
    let geom = get_roll_geom();
    let weights = ScoringWeights::default();

    // Index(0) -> Middle(1) = 1 -> 2 (Increasing Finger Index) => Roll Out
    let res_out = analyze_interaction(&geom, 0, 1, &weights);
    assert!(res_out.is_roll_out, "Index -> Middle should be Roll Out");
    assert!(!res_out.is_roll_in, "Index -> Middle should NOT be Roll In");

    // Ring(2) -> Middle(1) = 3 -> 2 (Decreasing Finger Index) => Roll In
    let res_in = analyze_interaction(&geom, 2, 1, &weights);
    assert!(res_in.is_roll_in, "Ring -> Middle should be Roll In");
}

#[test]
fn test_trigram_flow_detection() {
    let geom = get_roll_geom();
    let idx = &geom.keys[0]; // Finger 1
    let mid = &geom.keys[1]; // Finger 2
    let rng = &geom.keys[2]; // Finger 3

    // 1. Inward Roll (Pinky-ish direction: 3->2->1)
    // Ring -> Middle -> Index
    let flow_in = analyze_flow(rng, mid, idx);
    assert!(
        flow_in.is_inward_roll,
        "Ring->Mid->Index should be Inward Roll"
    );
    assert!(!flow_in.is_redirect, "Ring->Mid->Index is not a redirect");

    // 2. Outward Roll (Index direction: 1->2->3)
    // Index -> Middle -> Ring
    let flow_out = analyze_flow(idx, mid, rng);
    assert!(
        flow_out.is_outward_roll,
        "Index->Mid->Ring should be Outward Roll"
    );

    // 3. Redirect (Index -> Ring -> Middle) (1 -> 3 -> 2)
    // Dir1: +2, Dir2: -1. Different signs.
    let flow_redir = analyze_flow(idx, rng, mid);
    assert!(
        flow_redir.is_redirect,
        "Index->Ring->Mid should be Redirect"
    );

    // 4. ABA (Index -> Middle -> Index)
    let flow_aba = analyze_flow(idx, mid, idx);
    assert!(flow_aba.is_redirect, "ABA is a redirect");
}
