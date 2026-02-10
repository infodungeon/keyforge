#[keyforge_testing_macros::kf_test]
mod integration_tests {
    use super::*;
    // libs/keyforge-infra/tests/infra_integration.rs

    use keyforge_adapter::loader::AssetLoader;
    use keyforge_boundary::SafePath;
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
        "meta": { "name": "Test Board", "author": "Test", "version": "1.0", "notes": "", "kb_type": "ortho" },
        "geometry": { 
            "keys": [{"index":0, "label": "A", "x":0.0, "y":0.0, "hand":0, "finger":1, "row": 0, "col": 0, "w": 1.0, "h": 1.0, "is_home": true, "is_stretch": false, "r": 0.0, "rx": 0.0, "ry": 0.0}], 
            "prime_slots": [0], "med_slots": [], "low_slots": [], "home_row": 0
        },
        "layouts": {}
    }"#;
        fs::write(kb_dir.join("test_kb.json"), kb_json).unwrap();

        let provider = FsProvider::new(SafePath::from_trusted_root_path(root.to_path_buf()));

        // 1. Load existing asset
        let res = provider
            .load::<keyforge_protocol::KeyboardDefinitionDto>("test_kb")
            .await;
        assert!(res.is_ok(), "Failed to load test_kb: {:?}", res.err());
        let def: keyforge_model::geometry::KeyboardDefinition =
            res.unwrap().content.as_ref().clone().into();
        assert_eq!(def.meta.name, "Test Board");

        // 2. Load missing asset
        let missing = provider
            .load::<keyforge_protocol::KeyboardDefinitionDto>("missing")
            .await;
        assert!(missing.is_err());
    }
}
