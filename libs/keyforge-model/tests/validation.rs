use keyforge_model::{
    Corpus, Rubric, SearchConfig, Layout, KeyCode, Validator,
    KeyboardGeometry, KeyNode, HandIndex, FingerIndex
};

#[test]
fn test_layout_validation() {
    // Duplicates
    let keys = vec![KeyCode(65), KeyCode(66), KeyCode(65)];
    assert!(Layout::try_from(keys).is_err());

    // Valid
    let keys = vec![KeyCode(65), KeyCode(66), KeyCode(67)];
    assert!(Layout::try_from(keys).is_ok());
}

#[test]
fn test_search_config_validation() {
    let invalid = SearchConfig::Annealing {
        steps: 0,
        start_temp: 100.0,
        end_temp: 0.01,
        seed: 42,
        patience: 500,
        reheats: 3,
        reheat_factor: 0.5,
    };
    assert!(invalid.validate().is_err());
}

#[test]
fn test_rubric_validation() {
    let mut r = Rubric::default();
    assert!(r.validate().is_ok());

    r.trigram_coverage = 1.5; // > 1.0
    assert!(r.validate().is_err());

    r.sfb_base = -10.0; // Negative penalty
    assert!(r.validate().is_err());
}

#[test]
fn test_corpus_validation() {
    let mut c = Corpus::default();
    assert!(c.validate().is_ok());

    c.char_freqs = vec![0; 10]; // Too short
    assert!(c.validate().is_err());
}

#[test]
fn test_keyboard_geometry_validation() {
    let mut geom = KeyboardGeometry::default();
    // Empty keys
    assert!(geom.validate().is_err());

    // Invalid Key (Hand > 1)
    geom.keys.push(KeyNode {
        hand: HandIndex(2).try_into().unwrap_or(HandIndex(0)), // HandIndex protects itself, but let's force bad data if possible or check logic
        ..Default::default()
    });
    // Actually HandIndex::try_from prevents creation of invalid HandIndex, 
    // so we test the geometry validation logic for other fields like dimensions.
    
    geom.keys[0].w = 0.0;
    assert!(geom.validate().is_err());
}