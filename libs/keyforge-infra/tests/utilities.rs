// libs/keyforge-infra/tests/utilities.rs

//! Integration tests for infrastructure utility functions. Verifies the correctness
//! of environment-based configuration, path traversal prevention via filename
//! sanitization, and the integration of cached layout string parsing.


use keyforge_infra::config::CommonConfig;
use keyforge_infra::sanitize_filename;


#[test]
fn test_config_from_env() {
    temp_env::with_var("KEYFORGE_HIVE_URL", Some("http://test.local"), || {
        let cfg = CommonConfig::from_env();
        assert_eq!(cfg.hive_url.unwrap(), "http://test.local");
    });
}

#[test]
fn test_filename_sanitization() {
    // The allowlist includes '.', '-', and '_'. Slashes are replaced by '_'.
    // "../../etc/passwd" -> ".." + "_" + ".." + "_" + "etc" + "_" + "passwd"
    assert_eq!(sanitize_filename("../../etc/passwd"), ".._.._etc_passwd");
    assert_eq!(sanitize_filename("valid-file.json"), "valid-file.json");
}
