// UPDATED: use keyforge_core
use keyforge_core::scorer::loader::{load_cost_matrix, load_ngrams};
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use tempfile::NamedTempFile;

fn get_valid_chars() -> HashSet<u8> {
    b"abcdefghijklmnopqrstuvwxyz.,/;".iter().cloned().collect()
}

// --- N-GRAM LOAD TESTS ---

#[test]
fn test_loader_parses_valid_ngrams() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "a\t100").unwrap();
    writeln!(file, "th\t200").unwrap();
    writeln!(file, "the\t300").unwrap();

    let valid = get_valid_chars();
    let reader = File::open(file.path()).unwrap();
    let raw = load_ngrams(reader, &valid, 1.0, 100, true).unwrap(); // max_trigrams = 100

    assert_eq!(raw.char_freqs[b'a' as usize], 100.0);
    assert_eq!(raw.bigrams.len(), 1);
    assert_eq!(raw.trigrams.len(), 1);
}

#[test]
fn test_loader_trigram_limit() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "abc\t1").unwrap();
    writeln!(file, "def\t2").unwrap();
    writeln!(file, "ghi\t3").unwrap();
    writeln!(file, "jkl\t4").unwrap();

    let valid = get_valid_chars();
    let reader = File::open(file.path()).unwrap();

    // Set limit to 2
    let raw = load_ngrams(reader, &valid, 1.0, 2, true).unwrap();

    assert_eq!(raw.trigrams.len(), 2, "Loader did not stop at limit");
    assert_eq!(raw.trigrams[0].0, b'a');
    assert_eq!(raw.trigrams[1].0, b'd');
}

#[test]
fn test_loader_handles_complex_tsv() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "-gram\t*/*\t1/*\t...").unwrap();
    writeln!(file, "E\t100\t...").unwrap();
    writeln!(file, "2-gram\t*/*\t2/*\t...").unwrap();
    writeln!(file, "TH\t200\t...").unwrap();
    writeln!(file, "3-gram\t*/*\t3/*\t...").unwrap();
    writeln!(file, "THE\t300\t...").unwrap();

    let valid = get_valid_chars();
    let reader = File::open(file.path()).unwrap();
    let raw = load_ngrams(reader, &valid, 1.0, 1000, true).unwrap();

    assert_eq!(raw.char_freqs[b'e' as usize], 100.0);
    assert_eq!(raw.bigrams.len(), 1);
    assert_eq!(raw.bigrams[0].0, b't');
    assert_eq!(raw.bigrams[0].1, b'h');
    assert_eq!(raw.trigrams.len(), 1);
}

#[test]
fn test_loader_handles_case_insensitivity() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "TH\t100").unwrap();
    let valid = get_valid_chars();
    let reader = File::open(file.path()).unwrap();
    let raw = load_ngrams(reader, &valid, 1.0, 100, true).unwrap();
    assert_eq!(raw.bigrams.len(), 1);
    assert_eq!(raw.bigrams[0].0, b't');
}

#[test]
fn test_loader_skips_invalid_chars() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "q$\t100").unwrap();
    let valid = get_valid_chars();
    let reader = File::open(file.path()).unwrap();
    let raw = load_ngrams(reader, &valid, 1.0, 100, true).unwrap();
    assert_eq!(raw.bigrams.len(), 0);
}

// --- COST MATRIX LOAD TESTS ---

#[test]
fn test_loader_parses_cost_matrix() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "From,To,Cost").unwrap();
    writeln!(file, "KeyQ,KeyW,1.5").unwrap();
    let reader = File::open(file.path()).unwrap();
    let raw = load_cost_matrix(reader, true).unwrap();
    assert_eq!(raw.entries.len(), 1);
    assert_eq!(raw.entries[0].2, 1.5);
}

#[test]
fn test_loader_cost_matrix_handles_whitespace() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "From,To,Cost").unwrap();
    writeln!(file, "KeyQ , KeyW , 1.5").unwrap(); // Spaces!
    let reader = File::open(file.path()).unwrap();
    let raw = load_cost_matrix(reader, true).unwrap();
    assert_eq!(raw.entries.len(), 1);
    assert_eq!(raw.entries[0].2, 1.5);
}

#[test]
fn test_loader_cost_matrix_skips_bad_lines() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "From,To,Cost").unwrap();
    writeln!(file, "KeyQ,KeyW,1.5").unwrap(); // Good
    writeln!(file, "Garbage").unwrap(); // Bad
    let reader = File::open(file.path()).unwrap();
    let raw = load_cost_matrix(reader, true).unwrap();
    assert_eq!(raw.entries.len(), 1);
}
