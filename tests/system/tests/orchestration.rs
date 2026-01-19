// tests/system/tests/orchestration.rs

use keyforge_testing::HermeticWorkspace;

#[test]
fn test_hermetic_workspace_bootstrap() {
    let ws = HermeticWorkspace::new().with_default_assets();

    // Verify system structure
    assert!(ws.root.join("system/config/keycodes.json").exists());
    assert!(ws.root.join("user/keyboards/test_kb.json").exists());
}
