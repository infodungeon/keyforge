#[keyforge_testing_macros::kf_test]
mod integration_tests {
    use super::*;
    // libs/keyforge-infra/tests/infra_integration.rs

    use keyforge_infra::AssetLoader;
    use keyforge_infra::FsProvider;
    use keyforge_model::KeyboardDefinition;
    use std::fs;

    #[tokio::test]
    async fn test_fs_provider_lifecycle() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();

        // Setup directory structure
        let kb_dir = root.join("user/keyboards");
        fs::create_dir_all(&kb_dir).unwrap();

        let kb_json = r#"{
        "meta": { "name": "Test Board" },
        "geometry": { 
            "keys": [{"index":0, "x":0.0, "y":0.0, "hand":0, "finger":1, "row": 0}], 
            "prime_slots": [0], "med_slots": [], "low_slots": [], "home_row": 0
        },
        "layouts": {}
    }"#;
        fs::write(kb_dir.join("test_kb.json"), kb_json).unwrap();

        let provider = FsProvider::new(root.to_path_buf());

        // 1. Load existing asset
        let res = provider.load::<KeyboardDefinition>("test_kb").await;
        assert!(res.is_ok(), "Failed to load test_kb: {:?}", res.err());
        assert_eq!(res.unwrap().meta.name, "Test Board");

        // 2. Load missing asset
        let missing = provider.load::<KeyboardDefinition>("missing").await;
        assert!(missing.is_err());
    }
}
