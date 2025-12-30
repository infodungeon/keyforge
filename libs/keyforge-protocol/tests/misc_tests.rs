use keyforge_protocol::config::{CorpusSource, ScoringWeights, SearchParams};
use keyforge_protocol::geometry::{KeyNode, KeyboardDefinition, KeyboardGeometry};
 
use keyforge_protocol::{
    CostMatrixSource, JobConfig, JobRequest, KeyConstraint,
};
use keyforge_protocol::types::KeyIndex;
use keyforge_protocol::Validator;
use std::str::FromStr;

#[test]
fn test_corpus_source_parsing() {
    let c = CorpusSource::from_str("rust:1.5").unwrap();
    assert_eq!(c.id, "rust");
    assert_eq!(c.weight, 1.5);
    assert!(CorpusSource::from_str("rust:xyz").is_err());
}

#[test]
fn test_key_constraint_parsing() {
    let c = KeyConstraint::from_str("  10  :  KC_A  ").unwrap();
    assert_eq!(c.index, KeyIndex(10));
    assert_eq!(c.key, "KC_A");
    assert!(KeyConstraint::from_str("10-KC_A").is_err());
    assert!(KeyConstraint::from_str("NaN:KC_A").is_err());
}

#[test]
fn test_job_config_conversion() {
    let req = JobRequest {
        version: 1,
        definition: KeyboardDefinition::default(),
        weights: ScoringWeights::default(),
        params: SearchParams::default(),
        pinned_keys: vec![KeyConstraint {
            index: KeyIndex(0),
            key: "A".into(),
        }],
        corpora: vec![CorpusSource::default()],
        cost_matrix: CostMatrixSource::default(),
        biometrics: vec![],
        parent_job_id: Some("parent".into()),
        baseline_score: Some(100.0),
        parents: vec!["p1".into()],
    };
    let config: JobConfig = req.clone().into();
    assert_eq!(config.pinned_keys.len(), 1);
    assert_eq!(config.parent_job_id, Some("parent".into()));
}

#[test]
fn test_search_params_validation() {
    let mut p = SearchParams::default();
    p.search_epochs = 0;
    assert!(p.validate().is_err());
}

#[test]
fn test_geometry_validation() {
    let mut geom = KeyboardGeometry::default();
    assert!(geom.validate().is_err());
    let mut k = KeyNode::default();
    k.w = 0.0;
    geom.keys = vec![k];
    assert!(geom.validate().is_err());
}

#[test]
fn test_job_request_validation() {
    let mut req = JobRequest {
        version: 1,
        definition: KeyboardDefinition::default(),
        weights: ScoringWeights::default(),
        params: SearchParams::default(),
        pinned_keys: vec![],
        corpora: vec![],
        cost_matrix: CostMatrixSource::default(),
        biometrics: vec![],
        parent_job_id: None,
        baseline_score: None,
        parents: vec![],
    };
    req.definition.geometry.keys = vec![KeyNode::default()];
    req.definition.geometry.prime_slots = vec![KeyIndex(0)];
    req.pinned_keys = vec![KeyConstraint {
        index: KeyIndex(5),
        key: "A".into(),
    }];
    assert!(req.validate().is_err());
}
