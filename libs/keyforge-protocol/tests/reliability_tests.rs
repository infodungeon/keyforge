use keyforge_protocol::config::{CorpusSource, ScoringWeights, SearchParams};
use keyforge_protocol::geometry::{KeyNode, KeyboardDefinition, KeyboardGeometry};
use keyforge_protocol::job::{JobIdError, JobIdentifier};
use keyforge_protocol::parsing::{parse_key, KeyAction};
use keyforge_protocol::protocol::{
    BiometricSample, CostMatrixSource, JobConfig, JobRequest, KeyConstraint,
};
use keyforge_protocol::types::{KeyIndex, HandIndex, FingerIndex};
use keyforge_protocol::Validator;
use std::str::FromStr;

#[test]
fn test_corpus_source_parsing() {
    let c = CorpusSource::from_str("rust:1.5").unwrap();
    assert_eq!(c.id, "rust");
    assert_eq!(c.weight, 1.5);

    let c2 = CorpusSource::from_str("rust").unwrap();
    assert_eq!(c2.weight, 1.0);

    assert!(CorpusSource::from_str("rust:xyz").is_err());
    assert!(CorpusSource::from_str("rust:-1.0").is_err());
    assert!(CorpusSource::from_str("rust:0.0").is_err());
    assert!(CorpusSource::from_str("rust:NaN").is_err());
}

#[test]
fn test_key_constraint_parsing() {
    let c = KeyConstraint::from_str("  10  :  KC_A  ").unwrap();
    assert_eq!(c.index, KeyIndex(10));
    assert_eq!(c.key, "KC_A");

    assert!(KeyConstraint::from_str("10-KC_A").is_err());
    assert!(KeyConstraint::from_str("NaN:KC_A").is_err());
    assert!(KeyConstraint::from_str("10:   ").is_err());
    assert!(KeyConstraint::from_str("").is_err());
}

#[test]
fn test_cost_matrix_source_display() {
    let pre = CostMatrixSource::Predefined("test.json".into());
    assert_eq!(format!("{}", pre), "test.json");

    let cust = CostMatrixSource::Custom("data".into());
    assert_eq!(format!("{}", cust), "<custom_content>");
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
    assert_eq!(config.baseline_score, Some(100.0));
}

#[test]
fn test_search_params_validation() {
    let mut p = SearchParams::default();
    p.search_epochs = 0;
    assert!(p.validate().is_err());
    p.search_epochs = 100;
    p.search_epochs = 1_000_001;
    assert!(p.validate().is_err());
    p.search_epochs = 100;
    p.search_steps = 0;
    assert!(p.validate().is_err());
    p.search_steps = 100;
    p.search_steps = 5_000_001;
    assert!(p.validate().is_err());
    p.search_steps = 100;
    p.opt_limit_fast = 0;
    assert!(p.validate().is_err());
    p.opt_limit_fast = 100;
    p.opt_limit_fast = 10_001;
    assert!(p.validate().is_err());
    p.opt_limit_fast = 100;
    p.opt_limit_slow = 50;
    assert!(p.validate().is_err());
    p.opt_limit_slow = 200;
    p.temp_min = -1.0;
    assert!(p.validate().is_err());
    p.temp_min = 0.1;
    p.temp_max = 2000.0;
    assert!(p.validate().is_err());
    p.temp_max = 100.0;
    p.temp_min = 200.0;
    assert!(p.validate().is_err());
    p.temp_min = 0.1;
    p.search_patience_threshold = -0.1;
    assert!(p.validate().is_err());
    p.search_patience_threshold = 1.1;
    assert!(p.validate().is_err());
}

#[test]
fn test_scoring_weights_validation() {
    let mut w = ScoringWeights::default();
    w.penalty_sfb_base = 200_000_000.0;
    assert!(w.validate().is_err());
    w.penalty_sfb_base = 100.0;
    w.penalty_scissor = -10.0;
    assert!(w.validate().is_err());
    w.penalty_scissor = 10.0;
    w.loader_trigram_limit = 100_000;
    assert!(w.validate().is_err());
    w.loader_trigram_limit = 1000;
    w.finger_penalty_scale = "1.0,1.0".into();
    assert!(w.validate().is_err());
    w.finger_penalty_scale = "1.0,1.0,1.0,1.0,abc".into();
    assert!(w.validate().is_err());
    w.finger_penalty_scale = "1.0,1.0,1.0,1.0,NaN".into();
    assert!(w.validate().is_err());
}

#[test]
fn test_geometry_validation() {
    let mut geom = KeyboardGeometry::default();
    assert!(geom.validate().is_err());

    let mut k = KeyNode::default();
    k.w = 0.0;
    geom.keys = vec![k];
    assert!(geom.validate().is_err());

    let mut k2 = KeyNode::default();
    k2.hand = HandIndex(2);
    geom.keys = vec![k2];
    assert!(geom.validate().is_err());

    let mut k3 = KeyNode::default();
    k3.finger = FingerIndex(5);
    geom.keys = vec![k3];
    assert!(geom.validate().is_err());

    let k_valid = KeyNode::default();
    geom.keys = vec![k_valid];
    geom.prime_slots = vec![KeyIndex(0)];
    geom.med_slots = vec![KeyIndex(0)];
    assert!(geom.validate().is_err());

    geom.med_slots = vec![];
    geom.prime_slots = vec![KeyIndex(1)];
    assert!(geom.validate().is_err());

    geom.keys.push(KeyNode::default());
    geom.prime_slots = vec![KeyIndex(0)];
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

    req.definition.geometry.keys = vec![KeyNode::default(); 201];
    assert!(req.validate().is_err());
    req.definition.geometry.keys = vec![KeyNode::default()];

    req.pinned_keys = vec![KeyConstraint { index: KeyIndex(0), key: "A".into() }; 201];
    assert!(req.validate().is_err());
    req.pinned_keys.clear();

    req.pinned_keys = vec![KeyConstraint { index: KeyIndex(5), key: "A".into() }];
    assert!(req.validate().is_err());
    req.pinned_keys.clear();

    req.biometrics = vec![BiometricSample { bigram: "ab".into(), ms: 10.0, timestamp: 0 }; 10_001];
    assert!(req.validate().is_err());
    req.biometrics.clear();

    req.cost_matrix = CostMatrixSource::Predefined("".into());
    assert!(req.validate().is_err());
    req.cost_matrix = CostMatrixSource::Custom("".into());
    assert!(req.validate().is_err());
    req.cost_matrix = CostMatrixSource::Custom("invalid_csv".into());
    assert!(req.validate().is_err());
}

#[test]
fn test_job_id_error() {
    let err = JobIdError::Serialize("fail".into());
    assert!(format!("{}", err).contains("fail"));
}

#[test]
fn test_job_identifier_legacy() {
    let geom = KeyboardGeometry::default();
    let weights = ScoringWeights::default();
    let params = SearchParams::default();
    let pins = vec![];
    let corpus = "default";
    let cost = CostMatrixSource::default();
    let _ = JobIdentifier::from_parts(&geom, &weights, &params, &pins, corpus, &cost);
}

#[test]
fn test_parsing_edge_cases() {
    let action = parse_key("MO(32)");
    assert!(matches!(action, KeyAction::Raw(s) if s == "MO(32)"));
    let long = "A".repeat(40);
    let action = parse_key(&long);
    assert!(matches!(action, KeyAction::Raw(s) if s.len() == 32));
}

#[test]
fn test_keyboard_definition_parse() {
    let json = r#"{ "meta": { "name": "Test" }, "geometry": { "keys": [], "prime_slots": [], "med_slots": [], "low_slots": [], "home_row": 1 } }"#;
    let def = KeyboardDefinition::parse(json, None).unwrap();
    assert_eq!(def.meta.name, "Test");

    let kle = r#"[{"name": "KLE"}, [{"a":7}, "A"]]"#;
    let def_kle = KeyboardDefinition::parse(kle, Some("Imported")).unwrap();
    assert_eq!(def_kle.meta.name, "Imported");
    assert_eq!(def_kle.geometry.keys.len(), 1);

    assert!(KeyboardDefinition::parse("invalid", None).is_err());
}
