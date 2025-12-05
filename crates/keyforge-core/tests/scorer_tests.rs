// ===== keyforge/crates/keyforge-core/tests/scorer_tests.rs =====
use keyforge_core::config::Config;
use keyforge_core::optimizer::{mutation, Replica};
use keyforge_core::scorer::Scorer;
use std::sync::Arc;
use tempfile::tempdir;

mod common;
use common::{create_geom, KeyBuilder};

fn get_mock_geom() -> keyforge_core::geometry::KeyboardGeometry {
    let keys = vec![
        KeyBuilder::new(0, 0).id("k0").finger(1).build(),
        KeyBuilder::new(0, 1).id("k1").finger(2).build(),
    ];
    create_geom(keys)
}

#[test]
fn test_delta_drift() {
    let geom = get_mock_geom();
    let mut config = Config::default();
    config.defs.tier_high_chars = "ab".to_string();

    let dir = tempdir().unwrap();
    let cost_path = dir.path().join("cost.csv");
    let corpus_dir = dir.path().join("corpus");
    std::fs::create_dir(&corpus_dir).unwrap();

    // Write Assets
    std::fs::write(&cost_path, "From,To,Cost\nk0,k1,10.0").unwrap();
    std::fs::write(corpus_dir.join("1grams.csv"), "c,f\na,100\nb,100").unwrap();
    std::fs::write(corpus_dir.join("2grams.csv"), "c1,c2,f\na,b,50").unwrap();
    std::fs::write(corpus_dir.join("3grams.csv"), "c1,c2,c3,f\n").unwrap();

    let scorer = Scorer::new(
        cost_path.to_str().unwrap(),
        corpus_dir.to_str().unwrap(),
        &geom,
        config,
        false,
    )
    .unwrap();

    let scorer_arc = Arc::new(scorer);

    let mut replica = Replica::new(scorer_arc.clone(), 100.0, Some(123), 100, 100, "");

    replica.layout = vec![b'a' as u16, b'b' as u16];
    replica.pos_map = mutation::build_pos_map(&replica.layout);

    let (base_score, left, total) = scorer_arc.score_full(&replica.pos_map, 100);
    replica.score = base_score + replica.imbalance_penalty(left);
    replica.left_load = left;
    replica.total_freq = total;

    let idx_a = 0;
    let idx_b = 1;

    let (d_score, d_load) = replica.calc_delta(idx_a, idx_b, 100);

    let old_imb = replica.imbalance_penalty(replica.left_load);
    let new_load = replica.left_load + d_load;
    let new_imb = replica.imbalance_penalty(new_load);

    let predicted_total = (replica.score - old_imb) + d_score + new_imb;

    replica.layout.swap(idx_a, idx_b);
    replica.pos_map = mutation::build_pos_map(&replica.layout);

    let (real_base, real_left, _) = scorer_arc.score_full(&replica.pos_map, 100);
    let real_total = real_base + replica.imbalance_penalty(real_left);

    let diff = (predicted_total - real_total).abs();
    assert!(
        diff < 0.001,
        "Delta Drift Detected! Predicted: {}, Real: {}, Diff: {}",
        predicted_total,
        real_total,
        diff
    );
}
