//! Integration tests for agent calibration lifecycle.

// apps/keyforge-agent/tests/calibration_integration.rs

use keyforge_agent::agent::calibration;
use keyforge_infra::{AssetManager, HiveClient};
use keyforge_infra::net::client::ClientConfig;
use std::fs;
use tempfile::tempdir;

#[tokio::test]
async fn test_calibration_lifecycle() {
    let dir = tempdir().unwrap();
    let data_root = dir.path().to_path_buf();
    
    // 1. Setup Mock Environment
    let kb_dir = data_root.join("keyboards");
    fs::create_dir_all(&kb_dir).unwrap();
    
    // Create dummy Corne definition
    let corne_json = r#"{
        "meta": { "name": "corne", "author": "foostan", "version": "1", "type": "split" },
        "geometry": {
            "keys": [
                {"index":0, "x":0.0, "y":0.0, "hand":0, "finger":1, "row":0, "col":0},
                {"index":1, "x":1.0, "y":0.0, "hand":0, "finger":2, "row":0, "col":1}
            ],
            "prime_slots": [], "med_slots": [], "low_slots": [], "home_row": 1
        },
        "layouts": { "default": "A B" }
    }"#;
    fs::write(kb_dir.join("corne.json"), corne_json).unwrap();

    // Mock Client (won't actually connect because file exists)
    let client = HiveClient::new(ClientConfig::default()).unwrap();
    let assets = AssetManager::new(client, data_root.clone());

    // 2. Run Calibration (First Run)
    let ips = calibration::calibrate(&assets, &data_root).await.expect("Calibration failed");
    assert!(ips > 0.0, "IPS should be positive");

    // 3. Verify Persistence
    let cal_file = data_root.join("user/calibration.json");
    assert!(cal_file.exists(), "Calibration file not created");
    
    let content = fs::read_to_string(&cal_file).unwrap();
    assert!(content.contains("ips"), "Invalid calibration file format");

    // 4. Run Calibration (Second Run - Should be fast/cached)
    let start = std::time::Instant::now();
    let ips2 = calibration::calibrate(&assets, &data_root).await.expect("Calibration 2 failed");
    assert!(start.elapsed().as_millis() < 100, "Should have used cached value");
    assert_eq!(ips, ips2, "Cached value mismatch");
}
