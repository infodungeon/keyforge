use keyforge_protocol::keycodes::{KeycodeDefinition, KeycodeRegistry};

#[test]
fn test_registry_lookup() {
    let defs = vec![
        KeycodeDefinition {
            code: 65,
            id: "KC_A".into(),
            label: "A".into(),
            aliases: vec!["A".into()],
        }
    ];
    let reg = KeycodeRegistry::new(defs);

    assert_eq!(reg.get_code("KC_A"), Some(65));
    assert_eq!(reg.get_code("A"), Some(65));
    assert_eq!(reg.get_code("a"), Some(65)); // Case insensitive
    assert_eq!(reg.get_code("B"), None);

    assert_eq!(reg.get_label(65), "A");
    assert_eq!(reg.get_label(66), "[66]"); // Fallback
}

#[test]
fn test_registry_defaults() {
    let reg = KeycodeRegistry::new_with_defaults();
    assert!(reg.get_code("KC_NO").is_some());
    assert!(reg.get_code("KC_TRNS").is_some());
}
