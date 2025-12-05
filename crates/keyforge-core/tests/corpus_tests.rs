// ===== keyforge/crates/keyforge-core/tests/corpus_tests.rs =====
use keyforge_core::corpus::generate_ngrams;

#[test]
fn test_corpus_generation() {
    let content = "The quick brown fox jumps over the lazy dog.";
    let tsv_output = generate_ngrams(content, 100);

    assert!(tsv_output.contains("e\t"), "Monogram missing");
    assert!(tsv_output.contains("th\t"), "Bigram missing");
}
