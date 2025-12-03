use keyforge_core::config::Config;
use keyforge_core::optimizer::{mutation, Replica};
use keyforge_core::scorer::loader::{load_cost_matrix, load_ngrams};
use std::collections::HashSet;
use std::io::Cursor;
use std::sync::Arc;

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
fn test_in_memory_loading() {
    let cost_data = "From,To,Cost\nk0,k1,10.0\n";
    let ngram_data = "ab\t100\n";

    let cursor_cost = Cursor::new(cost_data);
    let costs = load_cost_matrix(cursor_cost, false).expect("Cost load failed");
    assert_eq!(costs.entries.len(), 1);
    assert_eq!(costs.entries[0].2, 10.0);

    let cursor_ngram = Cursor::new(ngram_data);
    let valid: HashSet<u8> = b"ab".iter().cloned().collect();
    let ngrams = load_ngrams(cursor_ngram, &valid, 1.0, 100, false).expect("Ngram load failed");

    assert_eq!(ngrams.bigrams.len(), 1);
}

#[test]
fn test_delta_drift() {
    let geom = get_mock_geom();
    let mut config = Config::default();
    config.defs.tier_high_chars = "ab".to_string();

    let dir = tempfile::tempdir().unwrap();
    let cost_path = dir.path().join("cost.csv");
    let ngram_path = dir.path().join("ngrams.tsv");
    std::fs::write(&cost_path, "From,To,Cost\nk0,k1,10.0").unwrap();
    std::fs::write(&ngram_path, "ab\t100").unwrap();

    let scorer = keyforge_core::scorer::Scorer::new(
        cost_path.to_str().unwrap(),
        ngram_path.to_str().unwrap(),
        &geom,
        config,
        false,
    )
    .unwrap();

    let scorer_arc = Arc::new(scorer);

    let mut replica = Replica::new(
        scorer_arc.clone(),
        100.0,
        Some(123),
        // ARGS: temp, seed, fast, slow, pins (5 args after scorer)
        100,
        100,
        "",
    );

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

#[test]
fn test_pinning_constraints() {
    let geom = get_mock_geom();
    let mut config = Config::default();
    config.defs.tier_high_chars = "ab".to_string();

    let dir = tempfile::tempdir().unwrap();
    let cost_path = dir.path().join("cost.csv");
    let ngram_path = dir.path().join("ngrams.tsv");
    std::fs::write(&cost_path, "From,To,Cost\nk0,k1,10.0").unwrap();
    std::fs::write(&ngram_path, "ab\t100").unwrap();

    let scorer = keyforge_core::scorer::Scorer::new(
        cost_path.to_str().unwrap(),
        ngram_path.to_str().unwrap(),
        &geom,
        config,
        false,
    )
    .unwrap();

    let pinned_str = "0:a";

    let mut replica = Replica::new(Arc::new(scorer), 1000.0, Some(123), 100, 100, pinned_str);

    assert_eq!(replica.layout[0], b'a' as u16);
    assert_eq!(replica.layout[1], b'b' as u16);

    let (accepted, _steps) = replica.evolve(100);

    assert_eq!(accepted, 0);
    assert_eq!(replica.layout[0], b'a' as u16);
}
