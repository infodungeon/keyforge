#[keyforge_testing_macros::kf_test]
mod integration_tests {
    use super::*;
    // libs/keyforge-infra/tests/fs_integration.rs
    //
    // Integration tests for filesystem utilities (io, listing, lock).
    // These tests require tempfile/filesystem access and validate contract/wiring.

    use keyforge_infra::fs::listing::{
        list_corpora, list_cost_matrices, list_keyboards, list_keymap_extras,
    };
    use keyforge_infra::{atomic_write, read_to_string_limited, WorkspaceLock};
    use std::fs;

    // ============================================================================
    // IO Tests (from src/fs/io.rs)
    // ============================================================================

    /// Intent: Verify `atomic_write` creates parent directories and writes content atomically.
    /// Expected Result: File is created with correct content, updates work correctly.
    #[test]
    fn test_atomic_write() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("subdir/test.txt");

        // Success with directory creation
        atomic_write(&path, "hello").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "hello");

        // Success with update
        atomic_write(&path, "updated").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "updated");
    }

    /// Intent: Verify `read_to_string_limited` respects size limits.
    /// Expected Result: Returns content when under limit, errors when over limit.
    #[test]
    fn test_read_to_string_limited() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("test.txt");
        fs::write(&path, "hello world").unwrap();

        // Success
        let res = read_to_string_limited(&path, 100).unwrap();
        assert_eq!(res, "hello world");

        let res = read_to_string_limited(&path, 5);
        assert!(res.is_err());
        assert!(format!("{:?}", res.err()).contains("exceeds size limit"));
    }

    /// Intent: Verify `atomic_write` fails gracefully when parent is a file.
    /// Expected Result: Returns error when path is invalid.
    #[test]
    fn test_atomic_write_fail() {
        // Attempt to write to a path where parent is a file (invalid)
        let temp = tempfile::tempdir().unwrap();
        let file_path = temp.path().join("file");
        fs::write(&file_path, "not a dir").unwrap();

        let bad_path = file_path.join("blocked/test.txt");
        let res = atomic_write(&bad_path, "data");
        assert!(res.is_err());
    }

    // ============================================================================
    // Listing Tests (from src/fs/listing.rs)
    // ============================================================================

    /// Intent: Verify listing functions correctly discover assets across system/user directories.
    /// Expected Result: All asset types are discovered with correct IDs.
    #[test]
    fn test_listing_filters() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();

        // 1. Keyboards
        let sys_kb = root.join("system/keyboards/models");
        let user_kb = root.join("user/keyboards");
        fs::create_dir_all(&sys_kb).unwrap();
        fs::create_dir_all(&user_kb).unwrap();
        fs::write(sys_kb.join("sys.mpk.zst"), "").unwrap();
        fs::write(user_kb.join("user.json"), "").unwrap();

        let list = list_keyboards(root).unwrap();
        assert_eq!(list.len(), 2);
        assert!(list.contains(&"sys".into()));
        assert!(list.contains(&"user".into()));

        // 2. Corpora
        let sys_corp = root.join("system/corpora/en/std");
        let user_corp = root.join("user/corpora/custom");
        fs::create_dir_all(&sys_corp).unwrap();
        fs::create_dir_all(&user_corp).unwrap();
        fs::write(sys_corp.join("1grams.mpk.zst"), "").unwrap();
        fs::write(user_corp.join("1grams.json"), "").unwrap();

        let list = list_corpora(root).unwrap();
        assert_eq!(list.len(), 2);
        assert!(list.contains(&"en/std".into()));
        assert!(list.contains(&"custom".into()));

        // 3. Cost Matrices
        let sys_cm = root.join("system/weights");
        let user_cm = root.join("user/weights");
        fs::create_dir_all(&sys_cm).unwrap();
        fs::create_dir_all(&user_cm).unwrap();
        fs::write(sys_cm.join("cm_sys.mpk.zst"), "").unwrap();
        fs::write(user_cm.join("cm_user.json"), "").unwrap();

        let list = list_cost_matrices(root).unwrap();
        assert!(list.contains(&"cm_sys".into()));
        assert!(list.contains(&"cm_user".into()));

        // 4. Keymap Extras
        let sys_extra = root.join("system/keymap_extras");
        let user_extra = root.join("user/keymap_extras");
        fs::create_dir_all(&sys_extra).unwrap();
        fs::create_dir_all(&user_extra).unwrap();
        fs::write(sys_extra.join("extra_sys.mpk.zst"), "").unwrap();
        fs::write(user_extra.join("extra_user.json"), "").unwrap();

        let list = list_keymap_extras(root).unwrap();
        assert!(list.contains(&"extra_sys".into()));
        assert!(list.contains(&"extra_user".into()));
    }

    /// Intent: Verify listing functions return empty sets for empty directories.
    /// Expected Result: All listing functions return empty Vec for empty root.
    #[test]
    fn test_listing_empty_root() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();

        assert!(list_keyboards(root).unwrap().is_empty());
        assert!(list_corpora(root).unwrap().is_empty());
        assert!(list_cost_matrices(root).unwrap().is_empty());
        assert!(list_keymap_extras(root).unwrap().is_empty());
    }

    // ============================================================================
    // Lock Tests (from src/fs/lock.rs)
    // ============================================================================

    /// Intent: Verify `WorkspaceLock` provides mutual exclusion.
    /// Expected Result: Only one lock can be held at a time; released locks can be reacquired.
    #[test]
    fn test_workspace_lock_exclusivity() {
        let temp = tempfile::tempdir().unwrap();
        let lock_path = temp.path().join("workspace.lock");
        fs::File::create(&lock_path).unwrap();

        let lock_a = WorkspaceLock::acquire(&lock_path);
        assert!(lock_a.is_ok());

        let lock_b = WorkspaceLock::acquire(&lock_path);
        assert!(lock_b.is_err());

        drop(lock_a);

        let lock_c = WorkspaceLock::acquire(&lock_path);
        assert!(lock_c.is_ok());
    }

    /// Intent: Verify explicit lock release allows reacquisition.
    /// Expected Result: After `release()`, another process can acquire the lock.
    #[test]
    fn test_workspace_lock_release() {
        let temp = tempfile::tempdir().unwrap();
        let lock_path = temp.path().join("workspace.lock");
        fs::File::create(&lock_path).unwrap();

        let lock = WorkspaceLock::acquire(&lock_path).unwrap();
        lock.release().unwrap();

        let lock2 = WorkspaceLock::acquire(&lock_path);
        assert!(lock2.is_ok());
    }
}
