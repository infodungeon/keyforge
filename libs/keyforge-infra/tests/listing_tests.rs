use keyforge_infra::listing;

#[test]
fn list_cost_matrices_includes_mpk_zst_and_excludes_non_cost_weights() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();

    // Create expected directory structure
    std::fs::create_dir_all(root.join("system/weights")).unwrap();
    std::fs::create_dir_all(root.join("user/weights")).unwrap();

    // A real cost matrix can be either json or mpk.zst
    std::fs::write(root.join("system/weights/cost_matrix.mpk.zst"), b"dummy").unwrap();
    std::fs::write(root.join("user/weights/custom.json"), b"{\"entries\":[]}").unwrap();

    // These are *not* cost matrices and must be excluded
    std::fs::write(root.join("system/weights/ortho_split.mpk.zst"), b"dummy").unwrap();
    std::fs::write(root.join("system/weights/row_stagger.mpk.zst"), b"dummy").unwrap();
    std::fs::write(root.join("system/weights/testing.mpk.zst"), b"dummy").unwrap();

    let list = listing::list_cost_matrices(root).unwrap();

    assert!(list.contains(&"cost_matrix".to_string()));
    assert!(list.contains(&"custom".to_string()));
    assert!(!list.contains(&"ortho_split".to_string()));
    assert!(!list.contains(&"row_stagger".to_string()));
    assert!(!list.contains(&"testing".to_string()));
}
