// apps/keyforge-agent/tests/persistence.rs

//! Integration tests for agent WAL persistence and submission outbox. Verifies the
//! reliability of the Write-Ahead Log (WAL) in capturing optimization results to disk,
//! enabling crash recovery, and ensuring submissions are successfully synchronized to
//! the Hive when network connectivity is restored.


use keyforge_agent::agent::network::ResultOutbox;
use keyforge_infra::HiveClient;
use keyforge_protocol::ResultSubmission;
use tempfile::tempdir;
use std::time::Duration;
use tokio::fs;

#[tokio::test]
async fn test_wal_persistence_on_failure() {
    let dir = tempdir().unwrap();
    let data_root = dir.path().to_path_buf();
    let wal_dir = data_root.join("user/agent_wal");

    // Client pointing to nowhere to force failure
    let client = HiveClient::new(keyforge_infra::net::client::ClientConfig {
        api_url: "http://localhost:1".into(),
        asset_url: "http://localhost:1".into(),
        ..Default::default()
    }).unwrap();
    let outbox = ResultOutbox::new(client, data_root.clone(), 10);

    let submission = ResultSubmission {
        version: 1,
        job_id: "test-job".into(),
        layout: "a b c".into(),
        score: 10.5,
        node_id: "test-node".into(),
        timestamp: 123456789,
        nonce: 42,
        signature: None,
    };

    outbox.save_to_wal(&submission).unwrap();

    // Wait for async processing
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify WAL file exists
    let mut entries = fs::read_dir(wal_dir).await.unwrap();
    let mut found = false;
    while let Ok(Some(entry)) = entries.next_entry().await {
        if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
            found = true;
            break;
        }
    }
    assert!(found, "WAL file should have been created for failed submission");
}