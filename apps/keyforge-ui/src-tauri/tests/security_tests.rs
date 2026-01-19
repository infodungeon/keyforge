// apps/keyforge-ui/src-tauri/tests/security_tests.rs

//! Integration tests for safe file I/O in the UI application.


use ui_lib::commands::library::cmd_safe_write_file;
use tempfile::TempDir;

#[test]
fn test_safe_write_validation() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    // 1. Valid Write
    let valid_path = root.join("layout.json");
    let res = cmd_safe_write_file(valid_path.to_str().unwrap(), "{}", true);
    assert!(res.is_ok(), "Valid JSON write should succeed");

    // 2. Invalid Extension
    let invalid_ext = root.join("script.sh");
    let res = cmd_safe_write_file(
        invalid_ext.to_str().unwrap(),
        "echo hack",
        true,
    );
    assert!(res.is_err(), "Shell script write should fail");

    // 3. Path Traversal
    // Note: cmd_safe_write_file checks for ".." string
    let traversal = root.join("../outside.json");
    let res = cmd_safe_write_file(traversal.to_str().unwrap(), "{}", true);
    assert!(res.is_err(), "Path traversal should fail");
}
