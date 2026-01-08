// libs/keyforge-infra/tests/data_integrity.rs

//! Integration tests for production data integrity. Verifies that standard corpora
//! (e.g., en_std) and cost matrices satisfy the domain's structural requirements when
//! loaded from the real workspace data directory.


use keyforge_infra::FsProvider;
use keyforge_core::loader::AssetLoader;
use keyforge_model::config::CorpusSource;
use std::path::PathBuf;

fn get_real_data_provider() -> FsProvider {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap().join("data");
    FsProvider::new(root)
}

#[tokio::test]
async fn test_corpus_en_std_integrity() {
    let provider = get_real_data_provider();
    let sources = vec![CorpusSource { id: "text/en_std".to_string(), weight: 1.0, hash: None }];

    let corpus = provider.load_corpus(&sources).await.expect("Failed to load en_std");

    // 1. Check Synthetic Injection
    let space = corpus.char_freqs[' ' as usize];
    let enter = corpus.char_freqs['\n' as usize];
    let bksp = corpus.char_freqs['\x08' as usize];

    assert!(space > 1_000_000, "Space freq too low");
    assert!(enter > 0, "Enter key not injected");
    assert!(bksp > 0, "Backspace key not injected");

    // 2. Check Sorting (Critical for Binary Search)
    let is_sorted = corpus.bigrams.windows(2).all(|w| w[0].0 <= w[1].0);
    assert!(is_sorted, "Bigrams are not sorted!");
}

#[tokio::test]
async fn test_cost_matrix_integrity() {
    let provider = get_real_data_provider();
    let costs = provider.load_cost_matrix("cost_matrix").await.expect("Failed to load costs");

    assert!(!costs.entries.is_empty());
    
    // Check for a known transition
    let has_home_jump = costs.entries.iter().any(|e| e.from == "KeyJ" && e.to == "KeyH");
    assert!(has_home_jump, "Missing J->H transition");
}