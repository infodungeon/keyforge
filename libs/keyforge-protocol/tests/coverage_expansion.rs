use keyforge_protocol::config::{CorpusSource, ScoringWeights, SearchParams};
use keyforge_protocol::geometry::{KeyNode, KeyboardDefinition, KeyboardGeometry};
use keyforge_protocol::protocol::{CostMatrixSource, JobRequest, KeyConstraint};
use keyforge_protocol::Validator;
use std::str::FromStr;

// --- Geometry Validation Tests ---

#[test]
fn test_geometry_valid() {
    let geom = KeyboardGeometry {
        keys: vec![KeyNode::default()],
        prime_slots: vec![0],
        med_slots: vec![],
        low_slots: vec![],
        home_row: 1,
    };
    assert!(geom.validate().is_ok());
}

#[test]
fn test_geometry_empty_keys() {
    let geom = KeyboardGeometry {
        keys: vec![],
        prime_slots: vec![],
        med_slots: vec![],
        low_slots: vec![],
        home_row: 1,
    };
    assert!(geom.validate().is_err());
}

#[test]
fn test_geometry_too_many_keys() {
    let keys = vec![KeyNode::default(); 201];
    let geom = KeyboardGeometry {
        keys,
        prime_slots: vec![],
        med_slots: vec![],
        low_slots: vec![],
        home_row: 1,
    };
    assert!(geom.validate().is_err());
}

#[test]
fn test_geometry_overlapping_slots() {
    let geom = KeyboardGeometry {
        keys: vec![KeyNode::default()],
        prime_slots: vec![0],
        med_slots: vec![0], // Overlap
        low_slots: vec![],
        home_row: 1,
    };
    assert!(geom.validate().is_err());
}

#[test]
fn test_geometry_incomplete_slots() {
    let geom = KeyboardGeometry {
        keys: vec![KeyNode::default(), KeyNode::default()],
        prime_slots: vec![0],
        med_slots: vec![],
        low_slots: vec![], // Index 1 missing
        home_row: 1,
    };
    assert!(geom.validate().is_err());
}

#[test]
fn test_geometry_slot_out_of_bounds() {
    let geom = KeyboardGeometry {
        keys: vec![KeyNode::default()],
        prime_slots: vec![1], // Out of bounds (len is 1)
        med_slots: vec![],
        low_slots: vec![],
        home_row: 1,
    };
    assert!(geom.validate().is_err());
}

#[test]
fn test_geometry_invalid_dimensions() {
    let mut key = KeyNode::default();
    key.w = 0.0;
    let geom = KeyboardGeometry {
        keys: vec![key],
        prime_slots: vec![0],
        med_slots: vec![],
        low_slots: vec![],
        home_row: 1,
    };
    assert!(geom.validate().is_err());
}

#[test]
fn test_geometry_invalid_hand() {
    let mut key = KeyNode::default();
    key.hand = 2; // Max is 1
    let geom = KeyboardGeometry {
        keys: vec![key],
        prime_slots: vec![0],
        med_slots: vec![],
        low_slots: vec![],
        home_row: 1,
    };
    assert!(geom.validate().is_err());
}

#[test]
fn test_geometry_invalid_finger() {
    let mut key = KeyNode::default();
    key.finger = 5; // Max is 4
    let geom = KeyboardGeometry {
        keys: vec![key],
        prime_slots: vec![0],
        med_slots: vec![],
        low_slots: vec![],
        home_row: 1,
    };
    assert!(geom.validate().is_err());
}

// --- JobRequest / CostMatrix Validation Tests ---

fn valid_job_request() -> JobRequest {
    let geom = KeyboardGeometry {
        keys: vec![KeyNode::default()],
        prime_slots: vec![0],
        med_slots: vec![],
        low_slots: vec![],
        home_row: 1,
    };

    JobRequest {
        version: 1,
        definition: KeyboardDefinition {
            geometry: geom,
            ..Default::default()
        },
        weights: ScoringWeights::default(),
        params: SearchParams::default(),
        pinned_keys: vec![],
        corpora: vec![CorpusSource::default()],
        cost_matrix: CostMatrixSource::Predefined("cost.json".into()),
        biometrics: vec![],
        parent_job_id: None,
        baseline_score: None,
        parents: vec![],
    }
}

#[test]
fn test_job_cost_matrix_predefined_empty() {
    let mut req = valid_job_request();
    req.cost_matrix = CostMatrixSource::Predefined("   ".into());
    assert!(req.validate().is_err());
}

#[test]
fn test_job_cost_matrix_custom_empty() {
    let mut req = valid_job_request();
    req.cost_matrix = CostMatrixSource::Custom("   ".into());
    assert!(req.validate().is_err());
}

#[test]
fn test_job_cost_matrix_custom_invalid_csv() {
    let mut req = valid_job_request();
    req.cost_matrix = CostMatrixSource::Custom("not_csv".into());
    assert!(req.validate().is_err());
}

#[test]
fn test_job_pinned_keys_out_of_bounds() {
    let mut req = valid_job_request();
    // Geometry has 1 key (index 0)
    req.pinned_keys = vec![KeyConstraint {
        index: 1,
        key: "A".into(),
    }];
    assert!(req.validate().is_err());
}

#[test]
fn test_job_too_many_biometrics() {
    let mut req = valid_job_request();
    req.biometrics = vec![
        keyforge_protocol::BiometricSample {
            bigram: "ab".into(),
            ms: 10.0,
            timestamp: 0
        };
        10001
    ];
    assert!(req.validate().is_err());
}

// --- Parsing Tests ---

#[test]
fn test_key_constraint_parsing() {
    assert!(KeyConstraint::from_str("").is_err());
    assert!(KeyConstraint::from_str("no_colon").is_err());
    assert!(KeyConstraint::from_str("abc:A").is_err());
    assert!(KeyConstraint::from_str("1:").is_err()); // Empty key

    let c = KeyConstraint::from_str("1:A").unwrap();
    assert_eq!(c.index, 1);
    assert_eq!(c.key, "A");
}

#[test]
fn test_corpus_source_parsing() {
    assert!(CorpusSource::from_str("id:invalid").is_err());
    assert!(CorpusSource::from_str("id:0.0").is_err());
    assert!(CorpusSource::from_str("id:NaN").is_err());

    let c = CorpusSource::from_str("id:1.5").unwrap();
    assert_eq!(c.id, "id");
    assert_eq!(c.weight, 1.5);
}

// --- KLE & Definition Tests ---

#[test]
fn test_keyboard_definition_parse_json() {
    let json = r#"{
        "meta": { "name": "Test", "author": "Me" },
        "geometry": { "keys": [], "prime_slots": [], "med_slots": [], "low_slots": [], "home_row": 1 }
    }"#;
    let def = KeyboardDefinition::parse(json, None).unwrap();
    assert_eq!(def.meta.name, "Test");
}

#[test]
fn test_keyboard_definition_parse_kle() {
    let kle = r#"[
        {"name": "KLE Test"},
        [{"a":7}, "A", "B"]
    ]"#;
    let def = KeyboardDefinition::parse(kle, Some("Imported")).unwrap();
    assert_eq!(def.meta.name, "Imported");
    assert_eq!(def.geometry.keys.len(), 2);
}

#[test]
fn test_keyboard_definition_parse_invalid() {
    assert!(KeyboardDefinition::parse("invalid", None).is_err());
}

#[test]
fn test_kle_export() {
    let geom = KeyboardGeometry {
        keys: vec![KeyNode {
            x: 1.0,
            y: 2.0,
            w: 1.0,
            h: 1.0,
            ..Default::default()
        }],
        prime_slots: vec![],
        med_slots: vec![],
        low_slots: vec![],
        home_row: 1,
    };
    let json = keyforge_protocol::geometry::kle::to_kle_json(&geom).unwrap();
    assert!(json.contains("\"x\": 1.0"));
    assert!(json.contains("\"y\": 2.0"));
}
