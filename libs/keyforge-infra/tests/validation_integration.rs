use keyforge_infra::FsProvider;
use keyforge_core::loader::AssetLoader;
use keyforge_model::error::ForgeError;
use tokio::fs;

#[tokio::test]
async fn test_load_invalid_keyboard_fails_validation() {
    // Setup: Create a temp dir with an invalid keyboard (0 keys)
    let temp_dir = std::env::temp_dir().join("keyforge_test_validation");
    let user_kb_dir = temp_dir.join("user/keyboards");
    fs::create_dir_all(&user_kb_dir).await.unwrap();

    let invalid_json = r#"{
        "meta": { "name": "Bad Board", "author": "Test" },
        "geometry": {
            "keys": [], 
            "prime_slots": [],
            "med_slots": [],
            "low_slots": [],
            "home_row": 1
        }
    }"#;
    
    fs::write(user_kb_dir.join("bad.json"), invalid_json).await.unwrap();

    // Execute
    let provider = FsProvider::new(temp_dir.clone());
    let result = provider.load_keyboard("bad").await;

    // Verify
    match result {
        Err(ForgeError::InvalidData(msg)) => {
            assert!(msg.contains("must have at least one key"), "Unexpected error msg: {}", msg);
        }
        Err(e) => panic!("Expected InvalidData error, got: {:?}", e),
        Ok(_) => panic!("Should have failed validation"),
    }

    // Cleanup
    let _ = fs::remove_dir_all(temp_dir).await;
}
