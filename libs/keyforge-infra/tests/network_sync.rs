// libs/keyforge-infra/tests/network_sync.rs

//! Integration tests for infrastructure network synchronization. Simulates a standard
//! sync flow using `HiveClient` and `wiremock` to verify that assets are correctly
//! identified via manifests, hashed for integrity, and downloaded to the local
//! workspace.


use keyforge_infra::{HiveClient, run_sync};
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

#[tokio::test]
async fn test_sync_flow() {
    let mock_server = MockServer::start().await;
    let client = HiveClient::new(mock_server.uri(), None).unwrap();
    let temp = tempfile::tempdir().unwrap();

    // The correct SHA-256 hash of the string "content"
    let actual_hash = "ed7002b439e9ac845f22357d822bac1444730fbdb6016d3ec9432297b9ec9f73";

    // Mock Manifest: Tell the client that test.txt exists with the correct hash
    let manifest = format!(r#"{{ "files": {{ "test.txt": "{}" }} }}"#, actual_hash);
    Mock::given(method("GET")).and(path("/manifest"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::from_str::<serde_json::Value>(&manifest).unwrap()))
        .mount(&mock_server).await;

    // Mock File: Provide the actual content
    Mock::given(method("GET")).and(path("/data/system/test.txt"))
        .respond_with(ResponseTemplate::new(200).set_body_string("content"))
        .mount(&mock_server).await;

    let stats = run_sync(&client, temp.path()).await.expect("Sync failed");
    
    // Verify that the file was downloaded and no errors occurred
    assert_eq!(stats.downloaded, 1, "File should have been downloaded. Errors: {:?}", stats.errors);
    assert!(temp.path().join("system/test.txt").exists());
}