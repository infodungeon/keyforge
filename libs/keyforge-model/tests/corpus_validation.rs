use keyforge_model::Corpus;

#[test]
fn test_corpus_validation_default() {
    let c = Corpus::default();
    assert!(c.validate().is_ok());
}

#[test]
fn test_corpus_validation_bad_freqs_short() {
    let mut c = Corpus::default();
    c.char_freqs = vec![0; 10]; // Too short
    assert!(c.validate().is_err());
}

#[test]
fn test_corpus_validation_bad_freqs_long() {
    let mut c = Corpus::default();
    c.char_freqs = vec![0; 70000]; // Too long
    assert!(c.validate().is_err());
}

#[test]
fn test_corpus_validation_valid_mutation() {
    let mut c = Corpus::default();
    c.char_freqs['a' as usize] = 100;
    c.bigrams.push(('a' as u16, 'b' as u16, 50));
    assert!(c.validate().is_ok());
}
