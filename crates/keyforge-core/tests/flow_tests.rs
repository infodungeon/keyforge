use keyforge_core::config::ScoringWeights;
use keyforge_core::geometry::KeyboardGeometry;
use keyforge_core::scorer::flow::analyze_flow;
use keyforge_core::scorer::physics::analyze_interaction;

// UPDATED: Import common
mod common;
use common::{create_geom, KeyBuilder};

// Mock 3-key Geometry (Same Hand)
fn get_roll_geom() -> KeyboardGeometry {
    let keys = vec![
        // Index (Finger 1)
        KeyBuilder::new(1, 3)
            .id("idx")
            .hand(0)
            .finger(1)
            .pos(3.0, 1.0)
            .build(),
        // Middle (Finger 2)
        KeyBuilder::new(1, 2)
            .id("mid")
            .hand(0)
            .finger(2)
            .pos(2.0, 1.0)
            .build(),
        // Ring (Finger 3)
        KeyBuilder::new(1, 1)
            .id("rng")
            .hand(0)
            .finger(3)
            .pos(1.0, 1.0)
            .build(),
    ];
    create_geom(keys)
}

#[test]
fn test_bigram_roll_detection() {
    let geom = get_roll_geom();
    let weights = ScoringWeights::default();

    let res_out = analyze_interaction(&geom, 0, 1, &weights);
    assert!(res_out.is_roll_out);

    let res_in = analyze_interaction(&geom, 2, 1, &weights);
    assert!(res_in.is_roll_in);
}

#[test]
fn test_trigram_flow_detection() {
    let geom = get_roll_geom();
    let idx = &geom.keys[0];
    let mid = &geom.keys[1];
    let rng = &geom.keys[2];

    let flow_in = analyze_flow(rng, mid, idx);
    assert!(flow_in.is_inward_roll);

    let flow_out = analyze_flow(idx, mid, rng);
    assert!(flow_out.is_outward_roll);

    let flow_redir = analyze_flow(idx, rng, mid);
    assert!(flow_redir.is_redirect);

    let flow_aba = analyze_flow(idx, mid, idx);
    assert!(flow_aba.is_redirect);
}
