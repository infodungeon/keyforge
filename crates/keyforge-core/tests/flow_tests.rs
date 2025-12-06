use keyforge_core::config::ScoringWeights;
use keyforge_core::geometry::KeyboardGeometry;
use keyforge_core::scorer::flow::analyze_flow;
use keyforge_core::scorer::physics::analyze_interaction;

mod common;
use common::{create_geom, KeyBuilder};

fn get_roll_geom() -> KeyboardGeometry {
    let keys = vec![
        KeyBuilder::new(1, 3)
            .id("idx")
            .hand(0)
            .finger(1)
            .pos(3.0, 1.0)
            .build(), // Index
        KeyBuilder::new(1, 2)
            .id("mid")
            .hand(0)
            .finger(2)
            .pos(2.0, 1.0)
            .build(), // Middle
        KeyBuilder::new(1, 1)
            .id("rng")
            .hand(0)
            .finger(3)
            .pos(1.0, 1.0)
            .build(), // Ring
    ];
    create_geom(keys)
}

#[test]
fn test_bigram_roll_detection() {
    println!("\n=== TEST: Bigram Roll Mechanics ===");
    let geom = get_roll_geom();
    let weights = ScoringWeights::default();

    // 1. Outward Roll (Index -> Middle)
    // Left Hand: Index(1) -> Middle(2). 1 < 2.
    // Direction is AWAY from thumb (0).
    println!("Scenario 1: Index(1) -> Middle(2) [Left Hand]");
    let res_out = analyze_interaction(&geom, 0, 1, &weights);
    println!("  -> Finger {} to Finger {}", res_out.finger, 2); // k1=0 is idx(1)
    println!("  -> Is Roll Out? {}", res_out.is_roll_out);
    assert!(res_out.is_roll_out, "Failed to detect Outward Roll");

    // 2. Inward Roll (Ring -> Middle)
    // Left Hand: Ring(3) -> Middle(2). 3 > 2.
    // Direction is TOWARD thumb.
    println!("Scenario 2: Ring(3) -> Middle(2) [Left Hand]");
    let res_in = analyze_interaction(&geom, 2, 1, &weights);
    println!("  -> Is Roll In?  {}", res_in.is_roll_in);
    assert!(res_in.is_roll_in, "Failed to detect Inward Roll");

    println!("✅ Bigram Rolls Verified.");
}

#[test]
fn test_trigram_flow_detection() {
    println!("\n=== TEST: Trigram Flow Mechanics ===");
    let geom = get_roll_geom();
    let idx = &geom.keys[0];
    let mid = &geom.keys[1];
    let rng = &geom.keys[2];

    // 1. Inward Roll (Ring -> Middle -> Index)
    println!("Scenario 1: Ring(3) -> Mid(2) -> Idx(1)");
    let flow_in = analyze_flow(rng, mid, idx);
    println!("  -> Is Inward Roll? {}", flow_in.is_inward_roll);
    assert!(flow_in.is_inward_roll);

    // 2. Outward Roll (Index -> Middle -> Ring)
    println!("Scenario 2: Idx(1) -> Mid(2) -> Ring(3)");
    let flow_out = analyze_flow(idx, mid, rng);
    println!("  -> Is Outward Roll? {}", flow_out.is_outward_roll);
    assert!(flow_out.is_outward_roll);

    // 3. Redirect (Index -> Ring -> Middle)
    // Direction Change: 1->3 (+) then 3->2 (-)
    println!("Scenario 3: Idx(1) -> Ring(3) -> Mid(2)");
    let flow_redir = analyze_flow(idx, rng, mid);
    println!("  -> Is Redirect? {}", flow_redir.is_redirect);
    assert!(flow_redir.is_redirect);

    // 4. Skipgram (ABA pattern: Index -> Middle -> Index)
    println!("Scenario 4: Idx(1) -> Mid(2) -> Idx(1)");
    let flow_aba = analyze_flow(idx, mid, idx);
    println!("  -> Is Redirect? {}", flow_aba.is_redirect);
    assert!(
        flow_aba.is_redirect,
        "ABA should count as redirect/direction change"
    );

    println!("✅ Trigram Flows Verified.");
}
