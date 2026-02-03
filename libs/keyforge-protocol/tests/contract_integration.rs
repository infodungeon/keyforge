// libs/keyforge-protocol/tests/contract_integration.rs

#[keyforge_testing_macros::kf_test]
mod integration_tests {
    use super::*;
    use keyforge_model::geometry::KeyNode;
    use keyforge_model::types::{FingerIndex, HandIndex, KeyCode, RowIndex, SpatialUnit};
    use keyforge_protocol::KeyNodeDto;

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