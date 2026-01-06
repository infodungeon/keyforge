use keyforge_infra::{HiveClient, run_sync};
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

#[tokio::test]
async fn test_sync_flow() {
    let mock_server = MockServer::start().await;
    let client = HiveClient::new(mock_server.uri(), None).unwrap();
    let temp = tempfile::tempdir().unwrap();

    // Mock Manifest
    let manifest = r#"{ "files": { "test.txt": "hash123" } }"#;
    Mock::given(method("GET")).and(path("/manifest"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::from_str::<serde_json::Value>(manifest).unwrap()))
        .mount(&mock_server).await;

    // Mock File
    Mock::given(method("GET")).and(path("/data/system/test.txt"))
        .respond_with(ResponseTemplate::new(200).set_body_string("content"))
        .mount(&mock_server).await;

    let stats = run_sync(&client, temp.path()).await.expect("Sync failed");
    assert_eq!(stats.downloaded, 1);
    assert!(temp.path().join("system/test.txt").exists());
}