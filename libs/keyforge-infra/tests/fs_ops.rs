// libs/keyforge-infra/tests/fs_ops.rs

//! Integration tests for infrastructure filesystem operations. Verifies the reliability
//! of workspace initialization, the exclusivity of the `WorkspaceLock` for preventing
//! concurrent access, and the accuracy of recursive asset listing filters.


use keyforge_infra::{listing, WorkspaceLock, initialize_workspace, InitMode};
use std::fs;

#[test]
fn test_listing_filters() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("system/keyboards/models")).unwrap();
    fs::write(root.join("system/keyboards/models/test.mpk.zst"), "").unwrap();
    
    let list = listing::list_keyboards(root).unwrap();
    assert!(list.contains(&"test".to_string()));
}

#[test]
fn test_workspace_lock_exclusivity() {
    let temp = tempfile::tempdir().unwrap();
    let lock_path = temp.path().join("workspace.lock");
    
    // Fix: WorkspaceLock::acquire uses File::open, which requires the file to exist.
    fs::File::create(&lock_path).unwrap();
    
    // 1. Acquire Lock A
    let lock_a = WorkspaceLock::acquire(&lock_path);
    assert!(lock_a.is_ok());

    // 2. Try Acquire Lock B (Should Fail)
    let lock_b = WorkspaceLock::acquire(&lock_path);
    assert!(lock_b.is_err(), "Should not be able to double-lock");
    
    // 3. Release A
    drop(lock_a);
    
    // 4. Acquire Lock B (Should Succeed)
    let lock_c = WorkspaceLock::acquire(&lock_path);
    assert!(lock_c.is_ok());
}

#[test]
fn test_init_workspace_creates_dirs() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("new_workspace");
    
    // Fix: initialize_workspace performs mandatory validation of system assets.
    // We must provide dummy files for the required assets to pass validation.
    let sys_root = root.join("system");
    fs::create_dir_all(sys_root.join("config")).unwrap();
    fs::create_dir_all(sys_root.join("weights")).unwrap();
    fs::create_dir_all(sys_root.join("corpora/text/en_std")).unwrap();
    
    fs::write(sys_root.join("config/keycodes.json"), "").unwrap();
    fs::write(sys_root.join("weights/cost_matrix.json"), "").unwrap();
    fs::write(sys_root.join("corpora/text/en_std/1grams.json"), "").unwrap();

    // Should create directory structure and pass validation
    initialize_workspace(&root, InitMode::Create).unwrap();
    
    assert!(root.join("system/config").exists());
    assert!(root.join("user/keyboards").exists());
    assert!(root.join("user/agent_wal").exists());
}