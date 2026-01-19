// libs/keyforge-infra/tests/data_integrity.rs

use keyforge_core::loader::AssetLoader;
use keyforge_infra::FsProvider;
use std::path::PathBuf;

#[tokio::test]
async fn test_load_real_assets() {
    // Point to 'data' directory, not 'data/system'
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap().join("data");
    
    if !root.exists() {
        eprintln!("Skipping test: data directory not found at {:?}", root);
        return;
    }

    let provider = FsProvider::new(root);

    // 1. Load Keyboard (ID "szr35" -> system/keyboards/models/szr35.mpk.zst)
    let kb: std::sync::Arc<keyforge_model::KeyboardDefinition> = provider.load("szr35").await.expect("Failed to load szr35");
    assert!(kb.geometry.keys.len() > 0);

    // 2. Load Cost Model
    let costs: std::sync::Arc<keyforge_model::CostModel> = provider.load("cost_matrix").await.expect("Failed to load costs");
    assert!(costs.models.contains_key("model_a_row_staggered"));

    // 3. Load Keycodes
    let keycodes: std::sync::Arc<keyforge_model::keycodes::KeycodeRegistry> = provider.load("keycodes").await.expect("Failed to load keycodes");
    assert!(keycodes.get_code("A").is_some());
}
