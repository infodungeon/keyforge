// apps/keyforge-agent/tests/calibration_integration.rs

/// Integration tests for agent calibration lifecycle.
#[keyforge_testing_macros::kf_test]
mod tests {
    use keyforge_agent::agent::calibration;
    use keyforge_agent::models::CalibrationConfig;
    use keyforge_infra::net::client::ClientConfig;
    use keyforge_infra::{AssetManager, HiveClient};
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_calibration_lifecycle() {
        let dir = tempdir().unwrap();
        let data_root = dir.path().to_path_buf();

        // 1. Setup Mock Environment
        let user_kb_dir = data_root.join("user/keyboards");
        fs::create_dir_all(&user_kb_dir).unwrap();

        let client = HiveClient::new(ClientConfig {
            base_url: "http://localhost:3002".to_string(),
            api_key: "test-key".to_string(),
            timeout_sec: 10,
        });
        let asset_mgr = AssetManager::new(client, data_root.clone());

        // 2. Perform Calibration
        let ips = calibration::calibrate(&asset_mgr, &data_root, &CalibrationConfig::default())
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
            base_url: "http://localhost:3002".to_string(),
            api_key: "test-key".to_string(),
            timeout_sec: 1,
        });

        let asset_mgr = AssetManager::new(client, data_root.join("non-existent"));
        let res =
            calibration::calibrate(&asset_mgr, &data_root, &CalibrationConfig::default()).await;

        assert!(res.is_err());
    }
}
