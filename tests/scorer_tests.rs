use keyforge::config::Config;
use keyforge::geometry::{KeyNode, KeyboardGeometry};
use keyforge::optimizer::{mutation, Replica};
use keyforge::scorer::loader::{load_cost_matrix, load_ngrams};
use keyforge::scorer::setup;
use std::collections::HashSet;
use std::io::Cursor;
use std::sync::Arc;

// --- MOCK GEOMETRY ---
fn get_mock_geom() -> KeyboardGeometry {
    // 2 keys: Index 0, Index 1
    let keys = vec![
        KeyNode {
            id: "k0".to_string(),
            hand: 0,
            finger: 1,
            row: 0,
            col: 0,
            x: 0.0,
            y: 0.0,
            is_stretch: false,
        },
        KeyNode {
            id: "k1".to_string(),
            hand: 0,
            finger: 2,
            row: 0,
            col: 1,
            x: 1.0,
            y: 0.0,
            is_stretch: false,
        },
    ];
    KeyboardGeometry {
        keys,
        prime_slots: vec![],
        med_slots: vec![],
        low_slots: vec![],
        home_row: 0,
        finger_origins: [[(0.0, 0.0); 5]; 2],
    }
}

// --- UNIT TEST: IN-MEMORY LOADING ---
#[test]
fn test_in_memory_loading() {
    let cost_data = "From,To,Cost\nk0,k1,10.0\n";
    let ngram_data = "ab\t100\n";

    // 1. Test Cost Loader
    let cursor_cost = Cursor::new(cost_data);
    let costs = load_cost_matrix(cursor_cost, false).expect("Cost load failed");
    assert_eq!(costs.entries.len(), 1);
    assert_eq!(costs.entries[0].2, 10.0);

    // 2. Test Ngram Loader
    let cursor_ngram = Cursor::new(ngram_data);
    let valid: HashSet<u8> = b"ab".iter().cloned().collect();
    let ngrams = load_ngrams(cursor_ngram, &valid, 1.0, false).expect("Ngram load failed");
    assert_eq!(ngrams.bigrams.len(), 1);
}

// --- UNIT TEST: DELTA DRIFT ---
#[test]
fn test_delta_drift() {
    // Setup Scorer
    let geom = get_mock_geom();
    let mut config = Config::default();
    config.defs.tier_high_chars = "ab".to_string(); // Minimal chars

    // Write temp files because setup::build_scorer still expects paths
    // (We refactored loader.rs but setup.rs still wraps File::open for the main binary flow.
    // To truly unit test the Replica, we'd need to mock Scorer construction without files,
    // but for now we use tempfile for the Scorer init, then test Replica in memory).
    let dir = tempfile::tempdir().unwrap();
    let cost_path = dir.path().join("cost.csv");
    let ngram_path = dir.path().join("ngrams.tsv");
    std::fs::write(&cost_path, "From,To,Cost\nk0,k1,10.0").unwrap();
    std::fs::write(&ngram_path, "ab\t100").unwrap();

    let scorer = setup::build_scorer(
        cost_path.to_str().unwrap(),
        ngram_path.to_str().unwrap(),
        config.weights,
        config.defs,
        geom,
        false,
    )
    .unwrap();

    let scorer_arc = Arc::new(scorer);

    // Initialize Replica
    let mut replica = Replica::new(
        scorer_arc.clone(),
        100.0, // Temp
        Some(123),
        false,
        100, // Limit
        100,
    );

    // Force specific layout: k0='a', k1='b'
    // 'a'->97, 'b'->98
    // pos_map: 97->0, 98->1
    replica.layout = vec![b'a', b'b'];
    replica.pos_map = mutation::build_pos_map(&replica.layout);

    let (base_score, left, total) = scorer_arc.score_full(&replica.pos_map, 100);
    replica.score = base_score + replica.imbalance_penalty(left);
    replica.left_load = left;
    replica.total_freq = total;

    // Perform a Swap and Compare Delta vs Recalculation
    let idx_a = 0;
    let idx_b = 1;

    // 1. Calculate Delta
    let (d_score, d_load) = replica.calc_delta(idx_a, idx_b, 100);

    // Calculate Predicted Total Score
    // Total = (Base - Old_Imbalance) + Delta_Base + New_Imbalance
    let old_imb = replica.imbalance_penalty(replica.left_load);
    let new_load = replica.left_load + d_load;
    let new_imb = replica.imbalance_penalty(new_load);

    let predicted_total = (replica.score - old_imb) + d_score + new_imb;

    // 2. Perform Swap
    replica.layout.swap(idx_a, idx_b);
    replica.pos_map = mutation::build_pos_map(&replica.layout);

    // 3. Full Re-score
    let (real_base, real_left, _) = scorer_arc.score_full(&replica.pos_map, 100);
    let real_total = real_base + replica.imbalance_penalty(real_left);

    // 4. Assert
    let diff = (predicted_total - real_total).abs();
    assert!(
        diff < 0.001,
        "Delta Drift Detected! Predicted: {}, Real: {}, Diff: {}",
        predicted_total,
        real_total,
        diff
    );
}
