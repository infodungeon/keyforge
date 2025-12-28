use keyforge_adapter::conversion;
use keyforge_model::SearchConfig;
use keyforge_protocol::keycodes::{KeycodeDefinition, KeycodeRegistry};
use keyforge_protocol::{config, geometry, KeyConstraint};

// ===== Conversion Tests =====

#[test]
fn test_to_domain_keyboard() {
    let mut geo = geometry::KeyboardGeometry::default();
    geo.home_row = 1;
    geo.keys = vec![
        geometry::KeyNode {
            id: "Q".to_string(),
            hand: 0,
            finger: 0,
            row: 0,
            col: 0,
            x: 0.0,
            y: 0.0,
            ..Default::default()
        },
        geometry::KeyNode {
            id: "A".to_string(),
            hand: 0,
            finger: 0,
            row: 1,
            col: 0,
            x: 0.0,
            y: 1.0,
            ..Default::default()
        },
    ];

    let kb = conversion::to_domain_keyboard(&geo);
    assert_eq!(kb.keys.len(), 2);
    assert_eq!(kb.keys[0].label, "Q");
    assert_eq!(kb.keys[0].is_home, false);
    assert_eq!(kb.keys[1].label, "A");
    assert_eq!(kb.keys[1].is_home, true);
}

#[test]
fn test_to_domain_rubric() {
    let mut weights = config::ScoringWeights::default();
    weights.penalty_sfb_base = 100.0;
    weights.penalty_sfb_lateral = 50.0;
    weights.weight_lateral_travel = 2.0;
    weights.weight_vertical_travel =3.0;
    weights.penalty_redirect = 200.0;
    weights.bonus_bigram_roll_in = 10.0;
    weights.trigram_coverage = 0.5;
    weights.loader_trigram_limit = 1000;

    let rubric = conversion::to_domain_rubric(&weights);
    assert_eq!(rubric.sfb_base, 100.0);
    assert_eq!(rubric.sfb_lateral, 50.0);
    assert_eq!(rubric.travel_lat, 2.0);
    assert_eq!(rubric.travel_vert, 3.0);
    assert_eq!(rubric.redirect, 200.0);
    assert_eq!(rubric.roll_bonus, 10.0);
    assert_eq!(rubric.trigram_coverage, 0.5);
    assert_eq!(rubric.trigram_limit, 1000);
}

#[test]
fn test_to_domain_config() {
    let mut params = config::SearchParams::default();
    params.search_steps = 10000;
    params.temp_max = 100.0;
    params.temp_min = 0.1;
    params.search_patience = 500;
    params.reheats = 3;
    params.reheat_factor = 1.5;

    let cfg = conversion::to_domain_config(&params, 42);
    
    match cfg {
        SearchConfig::Annealing {
            steps,
            start_temp,
            end_temp,
            seed,
            patience,
            reheats,
            reheat_factor,
        } => {
            assert_eq!(steps, 10000);
            assert_eq!(start_temp, 100.0);
            assert_eq!(end_temp, 0.1);
            assert_eq!(seed, 42);
            assert_eq!(patience, 500);
            assert_eq!(reheats, 3);
            assert_eq!(reheat_factor, 1.5);
        }
    }
}

// ===== Constraint Resolution Tests =====

fn test_registry() -> KeycodeRegistry {
    let defs = vec![
        KeycodeDefinition {
            code: 65,
            id: "KC_A".to_string(),
            label: "A".to_string(),
            aliases: vec!["A".to_string()],
        },
        KeycodeDefinition {
            code: 66,
            id: "KC_B".to_string(),
            label: "B".to_string(),
            aliases: vec!["B".to_string()],
        },
        KeycodeDefinition {
            code: 100,
            id: "MO".to_string(),
            label: "MO".to_string(),
            aliases: vec![],
        },
    ];
    KeycodeRegistry::new(defs)
}

#[test]
fn test_resolve_constraints_known_keys() {
    let registry = test_registry();

    let constraints = vec![
        KeyConstraint {
            index: 0,
            key: "A".to_string(),
        },
        KeyConstraint {
            index: 2,
            key: "B".to_string(),
        },
    ];

    let result = conversion::resolve_constraints(&constraints, 5, &registry).unwrap();
    assert_eq!(result.len(), 5);
    assert_eq!(result[0], Some(65));
    assert_eq!(result[1], None);
    assert_eq!(result[2], Some(66));
}

#[test]
fn test_resolve_constraints_numeric_fallback() {
    let registry = KeycodeRegistry::new_with_defaults();
    let constraints = vec![KeyConstraint {
        index: 0,
        key: "123".to_string(),
    }];

    let result = conversion::resolve_constraints(&constraints, 5, &registry).unwrap();
    assert_eq!(result[0], Some(123));
}

#[test]
fn test_resolve_constraints_unknown_key() {
    let registry = KeycodeRegistry::new_with_defaults();
    let constraints = vec![KeyConstraint {
        index: 0,
        key: "UNKNOWN_KEY".to_string(),
    }];

    let result = conversion::resolve_constraints(&constraints, 5, &registry);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown key"));
}

#[test]
fn test_resolve_constraints_out_of_bounds() {
    let registry = KeycodeRegistry::new_with_defaults();
    let constraints = vec![KeyConstraint {
        index: 10,
        key: "A".to_string(),
    }];

    let result = conversion::resolve_constraints(&constraints, 5, &registry);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("out of bounds"));
}

// ===== Cost Matrix Resolution Tests =====

#[test]
fn test_resolve_cost_matrix() {
    let mut geo = geometry::KeyboardGeometry::default();
    geo.keys = vec![
        geometry::KeyNode {
            id: "Q".to_string(),
            ..Default::default()
        },
        geometry::KeyNode {
            id: "A".to_string(),
            ..Default::default()
        },
        geometry::KeyNode {
            id: "Z".to_string(),
            ..Default::default()
        },
    ];

    let raw_data = vec![
        ("Q".to_string(), "A".to_string(), 2.5),
        ("A".to_string(), "Z".to_string(), 1.5),
        ("UNKNOWN".to_string(), "Z".to_string(), 999.0),
    ];

    let overrides = conversion::resolve_cost_matrix(&raw_data, &geo);
    assert_eq!(overrides.len(), 2);
    assert_eq!(overrides[0], (0, 1, 2.5));
    assert_eq!(overrides[1], (1, 2, 1.5));
}

#[test]
fn test_resolve_cost_matrix_empty() {
    let geo = geometry::KeyboardGeometry::default();
    let raw_data = vec![];
    let overrides = conversion::resolve_cost_matrix(&raw_data, &geo);
    assert_eq!(overrides.len(), 0);
}

// ===== Layout Parsing Tests =====

#[test]
fn test_parse_layout_string_strict_basic() {
    let registry = test_registry();
    let result = conversion::parse_layout_string_strict("A B A", 4, &registry);
    assert!(result.is_ok());
    let layout = result.unwrap();
    assert_eq!(layout.keys[0], 65);
    assert_eq!(layout.keys[1], 66);
    assert_eq!(layout.keys[2], 65);
    assert_eq!(layout.keys[3], 0);
}

#[test]
fn test_parse_layout_string_strict_argument_stripping() {
    let registry = test_registry();
    let result = conversion::parse_layout_string_strict("MO(1) A", 3, &registry);
    assert!(result.is_ok());
    let layout = result.unwrap();
    assert_eq!(layout.keys[0], 100);
    assert_eq!(layout.keys[1], 65);
}

#[test]
fn test_parse_layout_string_strict_ascii_fallback() {
    let registry = test_registry();
    let result = conversion::parse_layout_string_strict("A x B", 3, &registry);
    assert!(result.is_ok());
    let layout = result.unwrap();
    assert_eq!(layout.keys[0], 65);
    assert_eq!(layout.keys[1], 120);
    assert_eq!(layout.keys[2], 66);
}

#[test]
fn test_parse_layout_string_strict_unknown_token_error() {
    let registry = test_registry();
    let result = conversion::parse_layout_string_strict("A UNKNOWN B", 3, &registry);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown key token"));
}

#[test]
fn test_parse_layout_string_strict_max_length() {
    let registry = test_registry();
    let long_string = "A ".repeat(100000);
    let result = conversion::parse_layout_string_strict(&long_string, 10, &registry);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("exceeds maximum length"));
}

#[test]
fn test_parse_layout_string_permissive_unknown_tokens() {
    let registry = test_registry();
    let layout = conversion::parse_layout_string_permissive("A UNKNOWN B", 4, &registry);
    assert_eq!(layout.keys[0], 65);
    assert_eq!(layout.keys[1], 0);
    assert_eq!(layout.keys[2], 66);
    assert_eq!(layout.keys[3], 0);
}

#[test]
fn test_parse_layout_string_permissive_non_ascii() {
    let registry = test_registry();
    let layout = conversion::parse_layout_string_permissive("A ñ B", 3, &registry);
    assert_eq!(layout.keys[0], 65);
    assert_eq!(layout.keys[1], 0);
    assert_eq!(layout.keys[2], 66);
}

#[test]
fn test_parse_layout_string_permissive_size_limit() {
    let registry = test_registry();
    let layout = conversion::parse_layout_string_permissive("A B A B A B", 3, &registry);
    assert_eq!(layout.keys.len(), 3);
}

#[test]
fn test_parse_layout_string_backwards_compat() {
    let registry = test_registry();
    let result = conversion::parse_layout_string("A B", 2, &registry);
    assert!(result.is_ok());
}
