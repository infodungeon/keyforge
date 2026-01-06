use keyforge_infra::config::CommonConfig;
use keyforge_infra::{sanitize_filename, parse_layout_string_permissive_cached};
use keyforge_model::keycodes::KeycodeRegistry;

#[test]
fn test_config_from_env() {
    temp_env::with_var("KEYFORGE_HIVE_URL", Some("http://test.local"), || {
        let cfg = CommonConfig::from_env();
        assert_eq!(cfg.hive_url.unwrap(), "http://test.local");
    });
}

#[test]
fn test_filename_sanitization() {
    assert_eq!(sanitize_filename("../../etc/passwd"), "______etc_passwd");
    assert_eq!(sanitize_filename("valid-file.json"), "valid-file.json");
}

#[test]
fn test_layout_parser() {
    // Mock registry
    let registry = KeycodeRegistry::default(); // Assuming default has basic keys or empty
    // Note: In a real test you might need to load a real registry or mock one if KeycodeRegistry allows it.
    // If KeycodeRegistry::default() is empty, parser returns KC_NO (0).
    
    let layout_str = "KC_A KC_B";
    let layout = parse_layout_string_permissive_cached(layout_str, 2, &registry).unwrap();
    
    assert_eq!(layout.keys.len(), 2);
}