// apps/keyforge-cli/tests/io.rs

#[keyforge_testing_macros::kf_test]
mod tests {
    #[path = "common/mod.rs"]
    mod common;

    use keyforge_testing::HermeticWorkspace;
    use std::fs;

    #[test]
    fn test_resolve_absolute() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("test.txt");
        fs::write(&file, "hello").unwrap();
        assert!(file.is_absolute());
    }

    // Remaining tests...
}