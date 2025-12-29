use keyforge_model::{Corpus, Rubric};
use serde_json;

#[test]
fn test_corpus_lifecycle() {
    // 1. Default Construction
    let mut c = Corpus::default();
    assert_eq!(c.char_freqs.len(), 65536, "Corpus should initialize full unicode frequency map");
    assert!(c.bigrams.is_empty());
    assert!(c.trigrams.is_empty());
    assert!(c.words.is_empty());

    // 2. Mutation
    c.char_freqs['a' as usize] = 100;
    c.bigrams.push(('a' as u16, 'b' as u16, 50));
    c.trigrams.push(('a' as u16, 'b' as u16, 'c' as u16, 10));
    c.words.push(("test".to_string(), 5));

    // 3. Serialization Round-trip
    let json = serde_json::to_string(&c).expect("Failed to serialize Corpus");
    let recovered: Corpus = serde_json::from_str(&json).expect("Failed to deserialize Corpus");

    // 4. Verification
    assert_eq!(recovered.char_freqs['a' as usize], 100);
    assert_eq!(recovered.bigrams.len(), 1);
    assert_eq!(recovered.bigrams[0], ('a' as u16, 'b' as u16, 50));
    assert_eq!(recovered.trigrams.len(), 1);
    assert_eq!(recovered.words.len(), 1);
    assert_eq!(recovered.words[0].0, "test");
}

#[test]
fn test_rubric_lifecycle() {
    // 1. Default Construction
    let r = Rubric::default();
    
    // Check key defaults to ensure physics engine gets sensible start values
    assert!(r.sfb_base > 0.0);
    assert!(r.travel_lat > 0.0);
    assert!(r.travel_vert > 0.0);
    assert_eq!(r.finger_effort.len(), 5);

    // 2. Serialization Round-trip
    let json = serde_json::to_string(&r).expect("Failed to serialize Rubric");
    let recovered: Rubric = serde_json::from_str(&json).expect("Failed to deserialize Rubric");

    // 3. Verification
    assert_eq!(r.sfb_base, recovered.sfb_base);
    assert_eq!(r.finger_effort, recovered.finger_effort);
}

#[test]
fn test_rubric_modification() {
    let mut r = Rubric::default();
    r.sfb_base = 1000.0;
    r.finger_effort[4] = 5.0; // Pinky penalty

    assert_eq!(r.sfb_base, 1000.0);
    assert_eq!(r.finger_effort[4], 5.0);
}
