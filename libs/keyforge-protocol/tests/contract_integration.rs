#[keyforge_testing_macros::kf_test]
// libs/keyforge-protocol/tests/contract_integration.rs
use keyforge_model::{types::HandIndex, KeyNode};
use keyforge_protocol::JobRequest;

#[test]
#[allow(clippy::unwrap_used)]
fn test_contract_compatibility() {
    let mut req = JobRequest::default();

    // Test conversion from domain model to DTO
    let node = KeyNode {
        index: 0,
        label: "A".to_string(),
        x: 0.0,
        y: 0.0,
        w: 1.0,
        h: 1.0,
        hand: HandIndex::new(0),
        ..Default::default()
    };

    req.config.definition.geometry.keys().push(node.into());
    req.config.definition.geometry.home_row() = 0;
    req.config
        .definition
        .geometry
        .prime_slots
        .push(keyforge_model::types::KeyIndex(0).into());

    let json = serde_json::to_string(&req).unwrap();
    let deserialized: JobRequest = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.config.definition.geometry.keys().len(), 1);
}
