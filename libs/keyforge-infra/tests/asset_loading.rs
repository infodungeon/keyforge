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

use keyforge_infra::FsProvider;
use keyforge_core::loader::AssetLoader;
use keyforge_model::error::ForgeError;
use std::fs;

#[tokio::test]
async fn test_load_valid_user_keyboard() {
    let temp = tempfile::tempdir().unwrap();
    let kb_dir = temp.path().join("user/keyboards");
    fs::create_dir_all(&kb_dir).unwrap();

    // Create a minimal valid keyboard with all mandatory fields
    let json = r#"{
        "meta": { "name": "Test Board", "author": "Unit Test" },
        "geometry": { 
            "keys": [{"index":0, "x":0.0, "y":0.0, "hand":0, "finger":1}], 
            "prime_slots": [0],
            "med_slots": [],
            "low_slots": [],
            "home_row": 2 
        }
    }"#;
    fs::write(kb_dir.join("test_kb.json"), json).unwrap();

    let provider = FsProvider::new(temp.path().to_path_buf());
    let kb = provider.load_keyboard("test_kb").await.expect("Should load valid json");
    
    assert_eq!(kb.meta.name, "Test Board");
    assert_eq!(kb.geometry.keys.len(), 1);
}

#[tokio::test]
async fn test_load_invalid_keyboard_fails_validation() {
    let temp = tempfile::tempdir().unwrap();
    let kb_dir = temp.path().join("user/keyboards");
    fs::create_dir_all(&kb_dir).unwrap();

    // Invalid: Empty keys array (passes deserialization but fails .validate())
    let json = r#"{
        "meta": { "name": "Bad", "author": "Test" },
        "geometry": { 
            "keys": [], 
            "prime_slots": [],
            "med_slots": [],
            "low_slots": [],
            "home_row": 2 
        }
    }"#;
    fs::write(kb_dir.join("bad.json"), json).unwrap();

    let provider = FsProvider::new(temp.path().to_path_buf());
    let res = provider.load_keyboard("bad").await;
    
    match res {
        Err(ForgeError::InvalidData(msg)) => assert!(msg.contains("at least one key")),
        _ => panic!("Should have failed validation"),
    }
}