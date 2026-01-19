// libs/keyforge-persistence/tests/repository_integration.rs

use keyforge_persistence::UserRepo;
use std::fs;

#[test]
fn test_user_repo_layout_lifecycle() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();

    // Ensure the expected user directory exists
    fs::create_dir_all(root.join("user")).unwrap();

    let repo = UserRepo::new(root.to_path_buf());

    // 1. Save a layout
    repo.save_layout("corne", "My Layout", "A B C").unwrap();

    // 2. Retrieve it
    let layouts = repo.get_layouts("corne");
    assert_eq!(layouts.get("My Layout").unwrap(), "A B C");

    // 3. Delete it
    repo.delete_layout("corne", "My Layout").unwrap();
    let after = repo.get_layouts("corne");
    assert!(after.is_empty());
}
