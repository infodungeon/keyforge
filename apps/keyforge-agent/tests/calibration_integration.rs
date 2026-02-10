// apps/keyforge-agent/tests/calibration_integration.rs

/// Integration tests for agent calibration lifecycle.
#[keyforge_testing_macros::kf_test]
mod tests {
    use keyforge_agent::agent::calibration;
    use keyforge_agent::models::CalibrationConfig;
    use keyforge_boundary::SafePath;
    use keyforge_infra::net::client::ClientConfig;
    use keyforge_infra::{AssetManager, HiveClient};
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_calibration_lifecycle() {
        let dir = tempdir().unwrap();
        let data_root = dir.path().to_path_buf();

        // 1. Setup Mock Environment
        let sys_kb_dir = data_root.join("system/keyboards");
        fs::create_dir_all(&sys_kb_dir).unwrap();
        let corne_json = r#"{
            "meta": { "name": "corne", "author": "foostan", "version": "1", "notes": "", "type": "split" },
            "geometry": {
                "keys": [
                    {"index":0, "label":"k0", "x":0, "y":0, "w":1, "h":1, "hand":0, "finger":1, "row":0, "col":0},
                    {"index":1, "label":"k1", "x":1, "y":0, "w":1, "h":1, "hand":0, "finger":2, "row":0, "col":1}
                ],
                "prime_slots": [0, 1],
                "med_slots": [],
                "low_slots": [],
                "home_row": 0
            },
            "layouts": { "default": "A B" }
        }"#;
        fs::write(sys_kb_dir.join("corne.json"), corne_json).unwrap();

        let client = HiveClient::new(ClientConfig {
            api_url: "http://localhost:3002".to_string(),
            asset_url: "http://localhost:3001".to_string(),
            secret: Some("test-key".to_string()),
            timeout: std::time::Duration::from_secs(10),
            ..Default::default()
        })
        .expect("Failed to create mock client");
        let asset_mgr =
            AssetManager::new(client, SafePath::from_trusted_root_path(data_root.clone()));

        // 2. Perform Calibration
        let ips = calibration::calibrate(
            &asset_mgr,
            &SafePath::from_trusted_root_path(data_root.clone()),
            &CalibrationConfig::default(),
        )
        .await
        .unwrap();

        // 3. Verify
        assert!(ips > 0.0);
    }

    #[tokio::test]
    async fn test_calibration_io_resilience() {
        let dir = tempdir().unwrap();
        let data_root = dir.path().to_path_buf();

        // Ensure invalid root handled gracefully
        let client = HiveClient::new(ClientConfig {
            api_url: "http://localhost:3002".to_string(),
            asset_url: "http://localhost:3001".to_string(),
            secret: Some("test-key".to_string()),
            timeout: std::time::Duration::from_secs(1),
            ..Default::default()
        })
        .expect("Failed to create mock client");

        let asset_mgr = AssetManager::new(
            client,
            SafePath::from_trusted_root_path(data_root.join("non-existent")),
        );
        let res = calibration::calibrate(
            &asset_mgr,
            &SafePath::from_trusted_root_path(data_root.clone()),
            &CalibrationConfig::default(),
        )
        .await;

        assert!(res.is_err());
    }
}
