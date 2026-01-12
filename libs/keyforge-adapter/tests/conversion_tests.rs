// libs/keyforge-adapter/tests/conversion_tests.rs

//! Integration tests for the KeyForge adapter layer. Verifies the translation between
//! external protocol entities and internal domain models, including robust validation
//! of layout string parsing (strict and permissive), keycode alias resolution, and
//! search parameter mapping.


use keyforge_adapter::conversion;
use keyforge_adapter::error::AdapterError;
use keyforge_model::{
    config::{ScoringWeights, SearchParams},
    constants::MAX_LAYOUT_DATA_LEN,
    geometry::{KeyboardGeometry, KeyNode},
    keycodes::{KeycodeDefinition, KeycodeRegistry},
    types::{ColIndex, FingerIndex, HandIndex, KeyCode, KeyIndex, RowIndex},
    KeyConstraint,
};

#[test]
fn test_to_domain_keynode_conversion() {
    // Since Protocol uses Model types, we construct a Model KeyNode
    let proto_key = KeyNode {
        index: 0,
        label: "A".to_string(),
        x: 10.0,
        y: 20.0,
        w: 1.0,
        h: 1.0,
        r: 0.0,
        rx: 0.0,
        ry: 0.0,
        hand: HandIndex(0),
        finger: FingerIndex(1),
        row: RowIndex(0),
        col: ColIndex(0),
        is_home: true,
        is_stretch: false,
    };

    let domain_key = conversion::to_domain_keynode(proto_key.clone());

    assert_eq!(domain_key.index, proto_key.index);
    assert_eq!(domain_key.label, proto_key.label);
    assert_eq!(domain_key.hand, proto_key.hand);
}

#[test]
fn test_to_domain_keyboard_conversion() {
    let proto_geo = KeyboardGeometry {
        keys: vec![
            KeyNode { index: 0, label: "A".to_string(), hand: HandIndex(0), finger: FingerIndex(1), row: RowIndex(0), col: ColIndex(0), x: 0.0, y: 0.0, w: 1.0, h: 1.0, r: 0.0, rx: 0.0, ry: 0.0, is_home: true, is_stretch: false, ..Default::default() },
            KeyNode { index: 1, label: "B".to_string(), hand: HandIndex(1), finger: FingerIndex(2), row: RowIndex(0), col: ColIndex(1), x: 1.0, y: 0.0, w: 1.0, h: 1.0, r: 0.0, rx: 0.0, ry: 0.0, is_home: false, is_stretch: false, ..Default::default() },
        ],
        prime_slots: vec![KeyIndex(0)],
        med_slots: vec![KeyIndex(1)],
        low_slots: vec![],
        home_row: 0,
    };

    let domain_keyboard = conversion::to_domain_keyboard(&proto_geo).expect("Failed to convert keyboard");

    assert_eq!(domain_keyboard.count(), 2);
    assert_eq!(domain_keyboard.home_row, 0);
    assert_eq!(domain_keyboard.keys.len(), 2);
}

#[test]
fn test_to_domain_rubric_conversion() {
    let proto_weights = ScoringWeights {
        penalty_sfb_base: 100.0,
        penalty_sfb_lateral: 50.0,
        weight_lateral_travel: 2.0,
        weight_vertical_travel: 1.0,
        finger_penalty_scale: [1.0, 1.0, 1.0, 1.2, 1.5],
        penalty_redirect: 30.0,
        bonus_bigram_roll_in: 20.0,
        loader_trigram_limit: 5000,
        ..Default::default()
    };

    let domain_rubric = conversion::to_domain_rubric(&proto_weights);

    assert_eq!(domain_rubric.sfb_base, 100.0);
    assert_eq!(domain_rubric.sfb_lateral, 50.0);
    assert_eq!(domain_rubric.trigram_limit, 5000);
}

#[test]
fn test_resolve_constraints_valid() {
    let kb_geo = KeyboardGeometry {
        keys: vec![
            KeyNode { index: 0, label: "A".to_string(), ..Default::default() },
            KeyNode { index: 1, label: "B".to_string(), ..Default::default() },
            KeyNode { index: 2, label: "C".to_string(), ..Default::default() },
        ],
        prime_slots: vec![KeyIndex(0)], med_slots: vec![KeyIndex(1)], low_slots: vec![KeyIndex(2)], home_row: 0,
    };
    let mut registry = KeycodeRegistry::new_with_defaults();
    registry.definitions.push(KeycodeDefinition { code: KeyCode(10), id: "KC_A".to_string(), label: "A".to_string(), aliases: vec![] });
    registry.definitions.push(KeycodeDefinition { code: KeyCode(11), id: "KC_B".to_string(), label: "B".to_string(), aliases: vec!["11".to_string()] });
    registry.rebuild_maps();

    let proto_constraints = vec![
        KeyConstraint { index: KeyIndex(0), key: "KC_A".to_string() },
        KeyConstraint { index: KeyIndex(1), key: "11".to_string() }, // Alias for B
    ];

    let result = conversion::resolve_constraints(&proto_constraints, kb_geo.keys.len(), &registry);
    assert!(result.is_ok());
    let pins = result.unwrap();
    assert_eq!(pins.len(), 3);
    assert_eq!(pins[0], Some(KeyCode(10)));
    assert_eq!(pins[1], Some(KeyCode(11)));
    assert_eq!(pins[2], None);
}

#[test]
fn test_resolve_constraints_out_of_bounds() {
    let kb_geo = KeyboardGeometry {
        keys: vec![
            KeyNode { index: 0, label: "A".to_string(), ..Default::default() },
        ],
        prime_slots: vec![KeyIndex(0)], med_slots: vec![], low_slots: vec![], home_row: 0,
    };
    let registry = KeycodeRegistry::new_with_defaults();

    let proto_constraints = vec![
        KeyConstraint { index: KeyIndex(1), key: "KC_A".to_string() }, // Index 1 is out of bounds
    ];

    let result = conversion::resolve_constraints(&proto_constraints, kb_geo.keys.len(), &registry);
    assert!(result.is_err());
    match result.unwrap_err() {
        AdapterError::Validation(msg) => assert!(msg.contains("out of bounds")),
        _ => panic!("Expected Validation error"),
    }
}

#[test]
fn test_resolve_constraints_unknown_token() {
    let kb_geo = KeyboardGeometry {
        keys: vec![
            KeyNode { index: 0, label: "A".to_string(), ..Default::default() },
        ],
        prime_slots: vec![KeyIndex(0)], med_slots: vec![], low_slots: vec![], home_row: 0,
    };
    let registry = KeycodeRegistry::new_with_defaults();

    let proto_constraints = vec![
        KeyConstraint { index: KeyIndex(0), key: "UNKNOWN_KEY".to_string() },
    ];

    let result = conversion::resolve_constraints(&proto_constraints, kb_geo.keys.len(), &registry);
    assert!(result.is_err());
    match result.unwrap_err() {
        AdapterError::UnknownToken(t) => assert_eq!(t, "UNKNOWN_KEY"),
        _ => panic!("Expected UnknownToken error"),
    }
}

#[test]
fn test_parse_layout_string_strict_valid() {
    let kb_geo = KeyboardGeometry {
        keys: vec![
            KeyNode { index: 0, label: "A".to_string(), ..Default::default() },
            KeyNode { index: 1, label: "B".to_string(), ..Default::default() },
            KeyNode { index: 2, label: "C".to_string(), ..Default::default() },
        ],
        prime_slots: vec![KeyIndex(0)], med_slots: vec![KeyIndex(1)], low_slots: vec![KeyIndex(2)], home_row: 0,
    };
    let mut registry = KeycodeRegistry::new_with_defaults();
    // Add aliases so "A" resolves to KC_A
    registry.definitions.push(KeycodeDefinition { code: KeyCode(10), id: "KC_A".to_string(), label: "A".to_string(), aliases: vec!["A".to_string()] });
    registry.definitions.push(KeycodeDefinition { code: KeyCode(11), id: "KC_B".to_string(), label: "B".to_string(), aliases: vec!["B".to_string()] });
    registry.definitions.push(KeycodeDefinition { code: KeyCode(12), id: "KC_MO".to_string(), label: "MO(1)".to_string(), aliases: vec!["MO(1)".to_string()] });
    registry.rebuild_maps();

    let layout_str = "A B MO(1) D E F G H I J K L M N O P Q R S T U V W X Y Z";
    let result = conversion::parse_layout_string_strict(layout_str, kb_geo.keys.len(), &registry);
    assert!(result.is_ok());
    let layout = result.unwrap();
    assert_eq!(layout.keys.len(), 3);
    assert_eq!(layout.keys[0], KeyCode(10)); // A
    assert_eq!(layout.keys[1], KeyCode(11)); // B
    assert_eq!(layout.keys[2], KeyCode(12)); // MO(1)
}

#[test]
fn test_parse_layout_string_strict_invalid_token() {
    let kb_geo = KeyboardGeometry {
        keys: vec![
            KeyNode { index: 0, label: "A".to_string(), ..Default::default() },
            KeyNode { index: 1, label: "B".to_string(), ..Default::default() }, // Size 2
        ],
        prime_slots: vec![KeyIndex(0)], med_slots: vec![], low_slots: vec![], home_row: 0,
    };
    let mut registry = KeycodeRegistry::new_with_defaults();
    registry.definitions.push(KeycodeDefinition { code: KeyCode(10), id: "KC_A".to_string(), label: "A".to_string(), aliases: vec!["A".to_string()] });
    registry.rebuild_maps();

    // "A" fills slot 0. "UNKNOWN_TOKEN" tries to fill slot 1.
    // If size was 1, it would stop after "A".
    let layout_str = "A UNKNOWN_TOKEN";
    let result = conversion::parse_layout_string_strict(layout_str, kb_geo.keys.len(), &registry);
    assert!(result.is_err());
    match result.unwrap_err() {
        AdapterError::UnknownToken(t) => assert_eq!(t, "UNKNOWN_TOKEN"),
        _ => panic!("Expected UnknownToken error"),
    }
}

#[test]
fn test_parse_layout_string_strict_too_long() {
    let kb_geo = KeyboardGeometry {
        keys: vec![
            KeyNode { index: 0, label: "A".to_string(), ..Default::default() },
        ],
        prime_slots: vec![KeyIndex(0)], med_slots: vec![], low_slots: vec![], home_row: 0,
    };
    let registry = KeycodeRegistry::new_with_defaults();

    let layout_str = "A ".repeat(MAX_LAYOUT_DATA_LEN + 1);
    let result = conversion::parse_layout_string_strict(&layout_str, kb_geo.keys.len(), &registry);
    assert!(result.is_err());
    match result.unwrap_err() {
        AdapterError::LayoutTooLong(len) => assert_eq!(len, MAX_LAYOUT_DATA_LEN),
        _ => panic!("Expected LayoutTooLong error"),
    }
}

#[test]
fn test_parse_layout_string_permissive_valid() {
    let kb_geo = KeyboardGeometry {
        keys: vec![
            KeyNode { index: 0, label: "A".to_string(), ..Default::default() },
            KeyNode { index: 1, label: "B".to_string(), ..Default::default() },
            KeyNode { index: 2, label: "C".to_string(), ..Default::default() },
        ],
        prime_slots: vec![KeyIndex(0)], med_slots: vec![KeyIndex(1)], low_slots: vec![KeyIndex(2)], home_row: 0,
    };
    let mut registry = KeycodeRegistry::new_with_defaults();
    registry.definitions.push(KeycodeDefinition { code: KeyCode(10), id: "KC_A".to_string(), label: "A".to_string(), aliases: vec!["A".to_string()] });
    registry.definitions.push(KeycodeDefinition { code: KeyCode(11), id: "KC_B".to_string(), label: "B".to_string(), aliases: vec!["B".to_string()] });
    registry.definitions.push(KeycodeDefinition { code: KeyCode(12), id: "KC_MO".to_string(), label: "MO(1)".to_string(), aliases: vec!["MO(1)".to_string()] });
    registry.rebuild_maps();

    let layout_str = "A B MO(1) UNKNOWN_TOKEN Z"; // UNKNOWN_TOKEN should become KC_NO (0)
    let layout = conversion::parse_layout_string_permissive(layout_str, kb_geo.keys.len(), &registry);
    
    assert_eq!(layout.keys.len(), 3);
    assert_eq!(layout.keys[0], KeyCode(10)); // A
    assert_eq!(layout.keys[1], KeyCode(11)); // B
    assert_eq!(layout.keys[2], KeyCode(12)); // MO(1)
}

#[test]
fn test_parse_layout_string_permissive_padding() {
    let kb_geo = KeyboardGeometry {
        keys: vec![
            KeyNode { index: 0, label: "A".to_string(), ..Default::default() },
            KeyNode { index: 1, label: "B".to_string(), ..Default::default() },
        ],
        prime_slots: vec![KeyIndex(0)], med_slots: vec![KeyIndex(1)], low_slots: vec![], home_row: 0,
    };
    let registry = KeycodeRegistry::new_with_defaults();

    let layout_str = "A"; // Only one token provided
    let layout = conversion::parse_layout_string_permissive(layout_str, kb_geo.keys.len(), &registry);
    
    assert_eq!(layout.keys.len(), 2);
    // "A" is not in registry, and the ASCII backdoor is removed, so it should be KC_NO (0)
    assert_eq!(layout.keys[0], KeyCode(0)); 
    // Padded with KC_NO (0)
    assert_eq!(layout.keys[1], KeyCode(0)); 
}

#[test]
fn test_to_domain_rubric_conversion_defaults() {
    let proto_weights = ScoringWeights::default();
    let domain_rubric = conversion::to_domain_rubric(&proto_weights);

    // Default sfb_base is not 0.0 in the model constants.
    assert!(domain_rubric.sfb_base > 0.0);
    assert!(domain_rubric.trigram_limit > 0);
}

#[test]
fn test_to_domain_config_conversion() {
    let proto_params = SearchParams {
        search_steps: 100_000,
        temp_max: 20.0,
        temp_min: 0.005,
        search_patience: 500,
        reheats: 3,
        reheat_factor: 0.5,
        ..Default::default()
    };
    let seed = 42;
    let domain_config = conversion::to_domain_config(&proto_params, seed);

    match domain_config {
        keyforge_model::SearchConfig::Annealing { steps, start_temp, end_temp, seed: s, patience, reheats, reheat_factor } => {
            assert_eq!(steps, 100_000);
            assert_eq!(start_temp, 20.0);
            assert_eq!(end_temp, 0.005);
            assert_eq!(s, 42);
            assert_eq!(patience, 500);
            assert_eq!(reheats, 3);
            assert_eq!(reheat_factor, 0.5);
        }
    }
}
