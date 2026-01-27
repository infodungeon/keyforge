#[keyforge_testing_macros::kf_test]
// tests/system/tests/orchestration.rs
use keyforge_testing::HermeticWorkspace;

#[tokio::test]
async fn test_hermetic_workspace_bootstrap() {
    let ws = HermeticWorkspace::new()
        .await
        .expect("setup failed")
        .with_default_assets()
        .await
        .expect("assets failed");

    // Verify system structure
    assert!(ws.root.join("system/config/keycodes.json").exists());
    assert!(ws.root.join("user/keyboards/test_kb.json").exists());
}
