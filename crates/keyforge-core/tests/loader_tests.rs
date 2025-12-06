use keyforge_core::scorer::loader::{load_corpus_bundle, load_cost_matrix};
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_loader_corpus_bundle() {
    let dir = tempdir().unwrap();

    // Create 1grams.csv
    let p1 = dir.path().join("1grams.csv");
    let mut f1 = File::create(p1).unwrap();
    writeln!(f1, "char,freq\na,100").unwrap();

    // Create 2grams.csv
    let p2 = dir.path().join("2grams.csv");
    let mut f2 = File::create(p2).unwrap();
    writeln!(f2, "c1,c2,freq\nt,h,200").unwrap();

    // Create 3grams.csv
    let p3 = dir.path().join("3grams.csv");
    let mut f3 = File::create(p3).unwrap();
    writeln!(f3, "c1,c2,c3,freq\nt,h,e,300").unwrap();

    // Load Bundle
    let bundle = load_corpus_bundle(dir.path(), 1.0, 100).unwrap();

    assert_eq!(bundle.char_freqs[b'a' as usize], 100.0);
    assert_eq!(bundle.bigrams.len(), 1);
    assert_eq!(bundle.bigrams[0], (b't', b'h', 200.0));
    assert_eq!(bundle.trigrams.len(), 1);
    assert_eq!(bundle.trigrams[0], (b't', b'h', b'e', 300.0));
}

#[test]
fn test_loader_cost_matrix() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("cost.csv");

    let mut f = File::create(&path).unwrap();
    writeln!(f, "From,To,Cost\nKeyQ,KeyW,1.5").unwrap();

    let data = load_cost_matrix(&path).unwrap();
    assert_eq!(data.entries.len(), 1);
    assert_eq!(data.entries[0].2, 1.5);
}
