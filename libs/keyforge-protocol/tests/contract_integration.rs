// libs/keyforge-protocol/tests/contract_integration.rs
<<<<<<< HEAD
=======
use keyforge_model::{
    types::{HandIndex, KeyIndex},
    KeyNode,
};
use keyforge_protocol::JobRequest;
>>>>>>> master

#[keyforge_testing_macros::kf_test]
mod integration_tests {
    use super::*;
    use keyforge_model::geometry::KeyNode;
    use keyforge_model::types::{FingerIndex, HandIndex, KeyCode, RowIndex, SpatialUnit};
    use keyforge_protocol::KeyNodeDto;

<<<<<<< HEAD
    #[test]
    fn test_keynode_dto_to_domain_mapping() {
        let node = KeyNode {
            index: 0,
            label: "A".to_string(),
            x: SpatialUnit::from_f32(0.0),
            y: SpatialUnit::from_f32(0.0),
            w: 1.0,
            h: 1.0,
            hand: HandIndex::new(0),
            finger: FingerIndex::new_unchecked(1),
            row: RowIndex::new(0),
            col: keyforge_model::types::ColIndex::new(0),
            is_home: true,
            is_stretch: false,
            r: 0.0,
            rx: SpatialUnit::from_f32(0.0),
            ry: SpatialUnit::from_f32(0.0),
        };

        let dto: KeyNodeDto = node.clone().into();
        assert_eq!(dto.label, node.label);
        assert_eq!(dto.x, 0.0);
    }
}
=======
    // Test conversion from domain model to DTO
    let node = KeyNode {
        index: KeyIndex(0),
        label: "A".to_string(),
        x: keyforge_model::types::SpatialUnit::from_f32(0.0),
        y: keyforge_model::types::SpatialUnit::from_f32(0.0),
        w: 1.0,
        h: 1.0,
        hand: HandIndex::new(0),
        ..Default::default()
    };

    req.config.definition.geometry.keys.push(node.into());
    req.config.definition.geometry.home_row = 0;
    req.config
        .definition
        .geometry
        .prime_slots
        .push(KeyIndex(0).into());

    let json = serde_json::to_string(&req).unwrap();
    let deserialized: JobRequest = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.config.definition.geometry.keys.len(), 1);
}
>>>>>>> master
