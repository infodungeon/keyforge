use keyforge_core::config::Config;
mod common;
use common::{create_geom, KeyBuilder};
use std::fs;
use std::fs::File;
use std::io::Write;

#[test]
fn test_scorer_handles_small_geometry() {
    // 1. Setup 3-Key Geometry using Builder
    let keys = vec![
        KeyBuilder::new(0, 0)
            .id("k1")
            .finger(1)
            .pos(0.0, 0.0)
            .build(),
        KeyBuilder::new(0, 1)
            .id("k2")
            .finger(2)
            .pos(1.0, 0.0)
            .build(),
        KeyBuilder::new(0, 2)
            .id("k3")
            .finger(3)
            .pos(2.0, 0.0)
            .build(),
    ];

    let geom = create_geom(keys);

    // 2. Mock Data Files (Directory Structure)
    let dir = tempfile::tempdir().unwrap();
    let cost_path = dir.path().join("small_cost.csv");
    let corpus_dir = dir.path().join("small_corpus");
    fs::create_dir(&corpus_dir).unwrap();

    // Cost Matrix
    {
        let mut f = File::create(&cost_path).unwrap();
        writeln!(f, "From,To,Cost").unwrap();
    }

    // Corpus: 1grams.csv
    {
        let mut f = File::create(corpus_dir.join("1grams.csv")).unwrap();
        writeln!(f, "char,freq").unwrap();
        writeln!(f, "a,100").unwrap();
        writeln!(f, "b,100").unwrap();
        writeln!(f, "c,100").unwrap();
    }

    // Corpus: 2grams.csv
    {
        let mut f = File::create(corpus_dir.join("2grams.csv")).unwrap();
        writeln!(f, "char1,char2,freq").unwrap();
        writeln!(f, "a,b,100").unwrap();
    }

    // Corpus: 3grams.csv
    {
        let mut f = File::create(corpus_dir.join("3grams.csv")).unwrap();
        writeln!(f, "char1,char2,char3,freq").unwrap();
    }

    let mut config = Config::default();
    config.defs.tier_high_chars = "abc".to_string();
    config.weights.corpus_scale = 1.0; // Ensure scores aren't scaled to zero

    let scorer = keyforge_core::scorer::Scorer::new(
        cost_path.to_str().unwrap(),
        corpus_dir.to_str().unwrap(), // Pass Directory
        &geom,
        config,
        true,
    )
    .unwrap();

    assert_eq!(scorer.key_count, 3);

    // FIXED: Use Box<[u8; 65536]> instead of [u8; 256]
    let mut pos_map = Box::new([255u8; 65536]);
    pos_map[b'a' as usize] = 0;
    pos_map[b'b' as usize] = 1;
    pos_map[b'c' as usize] = 2;

    let (score, _, _) = scorer.score_full(&pos_map, 100);
    assert!(score > 0.0, "Score was {}, expected > 0.0", score);
}
