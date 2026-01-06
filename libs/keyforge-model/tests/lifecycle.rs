use keyforge_model::{Corpus, Rubric, Score};
use serde_json;

#[test]
fn test_corpus_serialization_roundtrip() {
    let mut c = Corpus::default();
    c.char_freqs['a' as usize] = 100;
    c.bigrams.push(('a' as u16, 'b' as u16, 50));
    
    let json = serde_json::to_string(&c).unwrap();
    let recovered: Corpus = serde_json::from_str(&json).unwrap();
    
    assert_eq!(recovered.char_freqs['a' as usize], 100);
    assert_eq!(recovered.bigrams[0], ('a' as u16, 'b' as u16, 50));
}

#[test]
fn test_rubric_defaults() {
    let r = Rubric::default();
    assert!(r.sfb_base > 0.0);
    assert_eq!(r.finger_effort.len(), 5);
}

#[test]
fn test_score_saturation() {
    let max = Score::MAX;
    let added = max + Score(100);
    assert_eq!(added, Score::MAX);

    let min = Score::MIN;
    let subbed = min - Score(100);
    assert_eq!(subbed, Score::MIN);
}