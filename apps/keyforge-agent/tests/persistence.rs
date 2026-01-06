// Copyright (c) 2025 KeyForge Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
    let client = HiveClient::new("http://localhost:1".into(), None).unwrap();
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

    outbox.try_send(submission).unwrap();

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