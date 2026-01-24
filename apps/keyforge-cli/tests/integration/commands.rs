// apps/keyforge-cli/tests/commands.rs

#[keyforge_testing_macros::kf_test]
mod tests {
    #[path = "common/mod.rs"]
    mod common;

    use keyforge_testing::HermeticWorkspace;
    use serde_json::Value;
    use std::fs;
    use std::process::Command;

    #[test]
    fn test_init_workspace() {
        let temp = tempfile::tempdir().unwrap();
        let bin = common::get_binary_path();

        let mut cmd = Command::new(&bin);
        cmd.arg("init").arg(temp.path()).arg("--defaults");

        let output = cmd.output().expect("Failed to run init");
        assert!(output.status.success());
        assert!(temp.path().join("keyforge.toml").exists());
    }

    // Remaining tests would continue here...
}