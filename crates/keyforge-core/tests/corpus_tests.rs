// ===== keyforge/crates/keyforge-core/tests/corpus_tests.rs =====
use keyforge_core::corpus::generate_ngrams;
use keyforge_core::scorer::loader::load_ngrams;
use std::collections::HashSet;
use std::io::Cursor;

#[test]
fn test_corpus_generation_and_loading_cycle() {
    // 1. Raw Content (Code + Prose mix)
    let content = r#"
        fn main() {
            println!("Hello, world!");
            let x = 10;
        }
        The quick brown fox jumps over the lazy dog.
    "#;

    // 2. Generate TSV (Top 100 ngrams)
    let tsv_output = generate_ngrams(content, 100);

    // 3. Verify Output Structure
    assert!(tsv_output.contains("e\t"), "Monogram 'e' missing");
    assert!(tsv_output.contains("th\t"), "Bigram 'th' missing");
    assert!(tsv_output.contains("the\t"), "Trigram 'the' missing");
    
    // Check punctuation handling
    assert!(tsv_output.contains(";\t"), "Semicolon missing");
    assert!(tsv_output.contains("()\t"), "Parens bigram missing");

    // 4. Load back using Scorer Loader to ensure compatibility
    let cursor = Cursor::new(tsv_output);
    let valid_chars: HashSet<u8> = b"abcdefghijklmnopqrstuvwxyz.,;\"'()!{}=_".iter().cloned().collect();
    
    // We use a lenient loader (debug=true logs errors but doesn't crash on some)
    let raw_ngrams = load_ngrams(cursor, &valid_chars, 1.0, 100, true)
        .expect("Generated TSV failed to load");

    // 5. Assert Logic
    // 'e' appears in "Hello", "let", "The", "over"
    let e_idx = b'e' as usize;
    assert!(raw_ngrams.char_freqs[e_idx] > 0.0, "Frequency for 'e' should be > 0");
    
    // 'the' appears twice (case-insensitive "The" and "the")
    let has_the = raw_ngrams.trigrams.iter().any(|(a, b, c, _)| {
        *a == b't' && *b == b'h' && *c == b'e'
    });
    assert!(has_the, "Trigram 'the' not found in parsed data");
}