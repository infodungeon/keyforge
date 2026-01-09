// apps/keyforge-agent/tests/config_test.rs

use keyforge_agent::models::{AgentConfig, PartialAgentConfig};

#[test]
fn test_config_merging() {
    let mut base = AgentConfig::default();
    
    // Simulate loading from file
    let partial = PartialAgentConfig {
        hive_url: Some("http://file-config.com".to_string()),
        cores: Some(8),
        calibration: Some(keyforge_agent::models::CalibrationConfig {
            key_count: 50,
            ..Default::default()
        }),
        ..Default::default()
    };

    base.merge(partial);

    assert_eq!(base.hive_url, "http://file-config.com");
    assert_eq!(base.cores, 8);
    assert_eq!(base.calibration.key_count, 50);
    // Check default preserved
    assert_eq!(base.network.timeout_seconds, 30);
}
