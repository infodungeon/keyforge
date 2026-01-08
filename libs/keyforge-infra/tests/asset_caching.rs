// libs/keyforge-infra/tests/asset_caching.rs

//! Integration tests for infrastructure asset caching and warmup. Verifies the
//! `CachingProvider`'s ability to recursively scan and index workspace assets,
//! ensuring that system manifests are correctly populated during the warmup phase.


use keyforge_infra::CachingProvider;
use keyforge_core::loader::AssetLoader;
use std::path::PathBuf;

#[tokio::test]
async fn test_system_warmup() {
    // Point to real workspace data
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap().join("data");

    let provider = CachingProvider::new(root);

    // 1. Run Warmup
    let result = provider.warm_all().await;
    assert!(result.is_ok(), "Warmup failed: {:?}", result.err());

    // 2. Verify Manifest
    let manifest = provider.get_manifest();
    assert!(manifest.is_some());
    assert!(manifest.unwrap().files.len() > 10, "Manifest too empty");

    // 3. Verify Cache Hits (Should be instant/in-memory)
    let kb = provider.load_keyboard("ansi_104").await;
    assert!(kb.is_ok());
    
    let kc = provider.load_keycodes("keycodes").await;
    assert!(kc.is_ok());
}