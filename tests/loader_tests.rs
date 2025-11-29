use keyforge::scorer::loader::{load_cost_matrix, load_ngrams};
use std::collections::HashSet;
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
    // Updated: Added .unwrap()
    let raw = load_ngrams(file.path().to_str().unwrap(), &valid, 1.0, true).unwrap();

    assert_eq!(raw.char_freqs[b'a' as usize], 100.0);
    assert_eq!(raw.bigrams.len(), 1);
    assert_eq!(raw.trigrams.len(), 1);
}

#[test]
fn test_loader_handles_complex_tsv() {
    // Simulation of your ngrams-all.tsv structure
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "-gram\t*/*\t1/*\t...").unwrap(); // Header 1
    writeln!(file, "E\t100\t...").unwrap(); // Monogram
    writeln!(file, "2-gram\t*/*\t2/*\t...").unwrap(); // Header 2
    writeln!(file, "TH\t200\t...").unwrap(); // Bigram
    writeln!(file, "3-gram\t*/*\t3/*\t...").unwrap(); // Header 3
    writeln!(file, "THE\t300\t...").unwrap(); // Trigram

    let valid = get_valid_chars();
    // Updated: Added .unwrap()
    let raw = load_ngrams(file.path().to_str().unwrap(), &valid, 1.0, true).unwrap();

    // Should skip headers (invalid chars '*' or '-')
    // Should match E, TH, THE
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
    // Updated: Added .unwrap()
    let raw = load_ngrams(file.path().to_str().unwrap(), &valid, 1.0, true).unwrap();
    assert_eq!(raw.bigrams.len(), 1);
    assert_eq!(raw.bigrams[0].0, b't');
}

#[test]
fn test_loader_skips_invalid_chars() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "q$\t100").unwrap();
    let valid = get_valid_chars();
    // Updated: Added .unwrap()
    let raw = load_ngrams(file.path().to_str().unwrap(), &valid, 1.0, true).unwrap();
    assert_eq!(raw.bigrams.len(), 0);
}

// --- COST MATRIX LOAD TESTS ---

#[test]
fn test_loader_parses_cost_matrix() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "From,To,Cost").unwrap();
    writeln!(file, "KeyQ,KeyW,1.5").unwrap();
    // Updated: Added .unwrap()
    let raw = load_cost_matrix(file.path().to_str().unwrap(), true).unwrap();
    assert_eq!(raw.entries.len(), 1);
    assert_eq!(raw.entries[0].2, 1.5);
}

#[test]
fn test_loader_cost_matrix_handles_whitespace() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "From,To,Cost").unwrap();
    writeln!(file, "KeyQ , KeyW , 1.5").unwrap(); // Spaces!
                                                  // Updated: Added .unwrap()
    let raw = load_cost_matrix(file.path().to_str().unwrap(), true).unwrap();
    assert_eq!(raw.entries.len(), 1);
    assert_eq!(raw.entries[0].2, 1.5);
}

#[test]
fn test_loader_cost_matrix_skips_bad_lines() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "From,To,Cost").unwrap();
    writeln!(file, "KeyQ,KeyW,1.5").unwrap(); // Good
    writeln!(file, "Garbage").unwrap(); // Bad
                                        // Updated: Added .unwrap()
    let raw = load_cost_matrix(file.path().to_str().unwrap(), true).unwrap();
    assert_eq!(raw.entries.len(), 1);
}
