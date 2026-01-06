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