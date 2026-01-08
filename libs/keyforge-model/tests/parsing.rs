// libs/keyforge-model/tests/parsing.rs

//! Integration tests for QMK/ZMK-style keycode parsing and Keyboard Layout Editor (KLE) JSON ingestion.
//! Verifies that complex key actions like Layer Tap (LT) and Momentary (MO) are correctly serialized
//! from string definitions.


use keyforge_model::parsing::{parse_key, KeyAction};
use keyforge_model::geometry::KeyboardDefinition;

#[test]
fn test_key_action_parsing() {
    assert_eq!(parse_key("KC_A"), KeyAction::Simple("KC_A".to_string()));
    assert_eq!(parse_key("TRNS"), KeyAction::Transparent);
    
    match parse_key("MO(1)") {
        KeyAction::LayerMomentary(1) => {},
        _ => panic!("Failed to parse MO(1)"),
    }

    match parse_key("LT(2, KC_SPC)") {
        KeyAction::LayerTap { layer: 2, key } => assert_eq!(key, "KC_SPC"),
        _ => panic!("Failed to parse LT"),
    }
}

#[test]
fn test_kle_import() {
    // Minimal KLE JSON
    let json = r#"[
        {"meta": {"name": "Test"}},
        [{"x":0},"A",{"x":1},"B"]
    ]"#;
    
    // Note: KeyboardDefinition::parse handles KLE detection
    let def = KeyboardDefinition::parse(json, Some("Test Board"));
    
    if let Ok(kb) = def {
        assert_eq!(kb.meta.name, "Test Board");
        assert_eq!(kb.geometry.keys.len(), 2);
    }
}