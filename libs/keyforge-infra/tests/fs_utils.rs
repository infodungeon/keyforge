use keyforge_infra::listing;
use std::fs;

#[test]
fn test_listing_filters_correctly() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    
    // Setup structure
    let sys_kb = root.join("system/keyboards/models");
    let usr_kb = root.join("user/keyboards");
    fs::create_dir_all(&sys_kb).unwrap();
    fs::create_dir_all(&usr_kb).unwrap();

    // Create files
    fs::write(sys_kb.join("sys_board.mpk.zst"), "").unwrap();
    fs::write(usr_kb.join("usr_board.json"), "").unwrap();
    fs::write(usr_kb.join("ignore_me.txt"), "").unwrap();

    let list = listing::list_keyboards(root).unwrap();
    
    assert!(list.contains(&"sys_board".to_string()));
    assert!(list.contains(&"usr_board".to_string()));
    assert!(!list.contains(&"ignore_me".to_string()));
}