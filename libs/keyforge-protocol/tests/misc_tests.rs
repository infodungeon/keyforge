// libs/keyforge-protocol/tests/misc_tests.rs

//! Miscellaneous integration tests for the protocol layer. Verifies the structural
//! integrity of protocol entities, including the conversion logic between `JobRequest`
//! and `JobConfig`, as well as rigorous validation of geometric constraints, pinned
//! key indices, and asset source names.


use keyforge_model::{
    CorpusSource, CostMatrixSource, KeyConstraint, KeyboardDefinition, ScoringWeights, SearchParams,
    Validator, KeyNode, KeyIndex
};
use keyforge_protocol::{JobConfig, JobRequest};

#[test]
fn test_job_config_conversion() {
    let req = JobRequest {
        version: 1,
        config: JobConfig {
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
        },
    };
    let config: JobConfig = req.clone().into();
    assert_eq!(config.pinned_keys.len(), 1);
    assert_eq!(config.parent_job_id, Some("parent".into()));
}

#[test]
fn test_job_request_validation() {
    let mut req = JobRequest {
        version: 1,
        config: JobConfig {
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
        },
    };
    
    // Setup valid minimal geometry
    req.config.definition.geometry.keys = vec![KeyNode::default()];
    req.config.definition.geometry.prime_slots = vec![KeyIndex(0)];

    // Test 1: Too many keys
    let original_keys = req.config.definition.geometry.keys.clone();
    req.config.definition.geometry.keys = vec![KeyNode::default(); 201];
    assert!(req.validate().is_err(), "Should reject > 200 keys");
    req.config.definition.geometry.keys = original_keys;

    // Test 2: Pinned key out of bounds
    req.config.pinned_keys = vec![KeyConstraint {
        index: KeyIndex(5), // Only 1 key exists (index 0)
        key: "A".into(),
    }];
    assert!(req.validate().is_err(), "Should reject out of bounds pin");
    req.config.pinned_keys.clear();

    // Test 3: Cost Matrix Empty
    req.config.cost_matrix = CostMatrixSource::Predefined("".into());
    assert!(req.validate().is_err(), "Should reject empty cost matrix name");
}