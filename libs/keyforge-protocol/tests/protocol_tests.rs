use keyforge_protocol::config::Config;
use keyforge_protocol::config::CorpusSource;
use keyforge_protocol::Validator;
use std::str::FromStr;

#[test]
fn test_config_defaults() {
    let c = Config::default();
    assert!(c.search.validate().is_ok());
    assert!(c.weights.validate().is_ok());
}

#[test]
fn test_corpus_source() {
    let c = CorpusSource::default();
    assert_eq!(c.id, "text/en_std");
    assert_eq!(c.weight, 1.0);

    let c1 = CorpusSource::from_str("rust").unwrap();
    assert_eq!(c1.id, "rust");
    assert_eq!(c1.weight, 1.0);

    let c2 = CorpusSource::from_str("rust:0.5").unwrap();
    assert_eq!(c2.id, "rust");
    assert_eq!(c2.weight, 0.5);

    assert!(CorpusSource::from_str("rust:invalid").is_err());
    assert!(CorpusSource::from_str("rust:0.0").is_err());
    assert!(CorpusSource::from_str("rust:-1.0").is_err());
    assert!(CorpusSource::from_str("rust:NaN").is_err());
}
