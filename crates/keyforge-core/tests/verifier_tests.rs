// ===== keyforge/crates/keyforge-core/tests/verifier_tests.rs =====
use keyforge_core::config::Config;
use keyforge_core::geometry::{KeyNode, KeyboardGeometry};
// FIXED: Removed unused KeycodeRegistry import
use keyforge_core::verifier::Verifier;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

fn create_mock_assets(
    dir: &std::path::Path,
) -> (std::path::PathBuf, std::path::PathBuf, std::path::PathBuf) {
    let cost_path = dir.join("cost.csv");
    let ngram_path = dir.join("ngrams.tsv");
    let kc_path = dir.join("keycodes.json");

    let mut f_cost = File::create(&cost_path).unwrap();
    writeln!(f_cost, "From,To,Cost\nk1,k2,10.0").unwrap();

    let mut f_ngram = File::create(&ngram_path).unwrap();
    writeln!(f_ngram, "a\t100\nb\t100\nab\t50").unwrap();

    // Minimal Registry (JSON)
    let mut f_kc = File::create(&kc_path).unwrap();
    writeln!(
        f_kc,
        r#"[
        {{"code": 97, "id": "KC_A", "label": "A", "aliases": []}},
        {{"code": 98, "id": "KC_B", "label": "B", "aliases": []}}
    ]"#
    )
    .unwrap();

    (cost_path, ngram_path, kc_path)
}

fn create_geometry() -> KeyboardGeometry {
    let keys = vec![
        KeyNode {
            id: "k1".into(),
            hand: 0,
            finger: 1,
            row: 0,
            col: 0,
            x: 0.0,
            y: 0.0,
            w: 1.0,
            h: 1.0,
            is_stretch: false,
        },
        KeyNode {
            id: "k2".into(),
            hand: 0,
            finger: 1,
            row: 0,
            col: 1,
            x: 1.0,
            y: 0.0,
            w: 1.0,
            h: 1.0,
            is_stretch: false,
        },
    ];
    let mut g = KeyboardGeometry {
        keys,
        prime_slots: vec![0, 1],
        med_slots: vec![],
        low_slots: vec![],
        home_row: 0,
        finger_origins: [[(0.0, 0.0); 5]; 2],
    };
    g.calculate_origins();
    g
}

#[test]
fn test_verifier_detects_drift() {
    let dir = tempdir().unwrap();
    let (cost, ngram, kc) = create_mock_assets(dir.path());
    let geom = create_geometry();
    let config = Config::default();

    // 1. Init Verifier
    let verifier = Verifier::new(
        cost.to_str().unwrap(),
        ngram.to_str().unwrap(),
        &geom,
        config,
        kc.to_str().unwrap(),
    )
    .expect("Failed to create verifier");

    // 2. Score Layout "AB"
    let layout = "A B";
    let details = verifier.score_details(layout.to_string());
    let true_score = details.layout_score;

    assert!(true_score > 0.0, "Score should be calculated");

    // 3. Verify Match
    let is_valid = verifier
        .verify(layout.to_string(), true_score, 0.1)
        .unwrap();
    assert!(is_valid, "Exact score should match");

    // 4. Verify Drift Detection
    let fake_score = true_score - 100.0;
    let is_valid_fake = verifier
        .verify(layout.to_string(), fake_score, 0.1)
        .unwrap();
    assert!(!is_valid_fake, "Drift should be detected");
}
