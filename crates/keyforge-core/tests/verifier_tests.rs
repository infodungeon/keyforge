use keyforge_core::config::Config;
use keyforge_core::geometry::{KeyNode, KeyboardGeometry};
use keyforge_core::verifier::Verifier;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

fn create_mock_assets(
    dir: &std::path::Path,
) -> (std::path::PathBuf, std::path::PathBuf, std::path::PathBuf) {
    let cost_path = dir.join("cost.csv");
    let corpus_dir = dir.join("corpus");
    let kc_path = dir.join("keycodes.json");

    std::fs::create_dir(&corpus_dir).unwrap();

    let mut f_cost = File::create(&cost_path).unwrap();
    writeln!(f_cost, "From,To,Cost\nk1,k2,10.0").unwrap();

    let mut f1 = File::create(corpus_dir.join("1grams.csv")).unwrap();
    writeln!(f1, "char,freq\na,100\nb,100").unwrap();

    let mut f2 = File::create(corpus_dir.join("2grams.csv")).unwrap();
    writeln!(f2, "char1,char2,freq\na,b,50").unwrap();

    let mut f3 = File::create(corpus_dir.join("3grams.csv")).unwrap();
    writeln!(f3, "char1,char2,char3,freq").unwrap();

    let mut f_kc = File::create(&kc_path).unwrap();
    writeln!(
        f_kc,
        r#"[
        {{"code": 97, "id": "KC_A", "label": "A", "aliases": []}},
        {{"code": 98, "id": "KC_B", "label": "B", "aliases": []}}
    ]"#
    )
    .unwrap();

    (cost_path, corpus_dir, kc_path)
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
    let (cost, corpus, kc) = create_mock_assets(dir.path());
    let geom = create_geometry();
    let config = Config::default();

    let verifier = Verifier::new(
        cost.to_str().unwrap(),
        corpus.to_str().unwrap(),
        &geom,
        config,
        kc.to_str().unwrap(),
    )
    .expect("Failed to create verifier");

    let layout = "A B";
    let details = verifier.score_details(layout.to_string());
    let true_score = details.layout_score;

    assert!(true_score > 0.0, "Score should be calculated");

    // FIXED: Removed extra argument
    let is_valid = verifier
        .verify(layout.to_string(), true_score, 0.1)
        .unwrap();
    assert!(is_valid, "Exact score should match");

    let fake_score = true_score - 100.0;
    // FIXED: Removed extra argument
    let is_valid_fake = verifier
        .verify(layout.to_string(), fake_score, 0.1)
        .unwrap();
    assert!(!is_valid_fake, "Drift should be detected");
}
