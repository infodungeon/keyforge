#[keyforge_testing_macros::kf_test]
mod integration_tests {
    use super::*;
    // libs/keyforge-model/tests/parsing_integration.rs
    //
    // Integration tests for parsing external data formats (KLE, layout strings).
    // These tests parse real or realistic external format data.

    use keyforge_model::geometry::KeyboardDefinition;

    /// Intent: Verify KLE JSON import correctly converts to `KeyboardDefinition`.
    /// Expected Result: Parses 2 keys from minimal KLE, preserves metadata from hint.
    #[test]
    fn test_kle_import_from_json() {
        let json = r#"[
        {"meta": {"name": "Test"}},
        [{"x":0},"A",{"x":1},"B"]
    ]"#;

        let def = KeyboardDefinition::parse(json, Some("Test Board"))
            .expect("KLE JSON should parse successfully");

        assert_eq!(def.meta.name, "Test Board");
        assert_eq!(def.geometry.keys.len(), 2);
        assert_eq!(def.geometry.keys[0].label, "A");
        assert_eq!(def.geometry.keys[1].label, "B");
    }

    /// Intent: Verify KLE import handles rotation properties.
    /// Expected Result: Rotation origin (rx, ry) and angle (r) parsed correctly.
    #[test]
    fn test_kle_import_with_rotation() {
        let json = r#"[
        [{"r": 15, "rx": 5, "ry": 5}, "A"]
    ]"#;

        let def = KeyboardDefinition::parse(json, None).expect("Rotated KLE should parse");

        assert_eq!(def.geometry.keys[0].r, 15.0);
        assert_eq!(def.geometry.keys[0].rx, 5.0);
        assert_eq!(def.geometry.keys[0].ry, 5.0);
    }

    /// Intent: Verify hand split detection for split keyboards.
    /// Expected Result: Keys before gap assigned Left, after gap assigned Right.
    #[test]
    fn test_kle_import_split_detection() {
        use keyforge_model::types::HandIndex;
        let json = r#"[["A", "B", {"x": 15}, "C"]]"#;

        let def = KeyboardDefinition::parse(json, None).expect("Split KLE should parse");

        assert_eq!(def.geometry.keys[0].hand, HandIndex::LEFT);
        assert_eq!(def.geometry.keys[1].hand, HandIndex::LEFT);
        assert_eq!(def.geometry.keys[2].hand, HandIndex::RIGHT);
    }
}
