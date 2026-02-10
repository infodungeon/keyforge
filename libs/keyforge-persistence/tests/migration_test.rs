#[keyforge_testing_macros::kf_test]
mod integration_tests {
    use super::*;
    use keyforge_boundary::SafePath;
    use keyforge_persistence::UserRepo;
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_user_layouts_migration() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();

        // 1. Create legacy user_layouts.json
        let legacy_path = root.join("user/user_layouts.json");
        fs::create_dir_all(legacy_path.parent().unwrap()).unwrap();

        let legacy_data = json!({
            "layouts": {
                "corne": {
                    "My Layout": "Q W E R T Y U I O P"
                },
                "ansi": {
                    "QWERTY": "q w e r t y u i o p"
                }
            }
        });

        fs::write(&legacy_path, serde_json::to_string(&legacy_data).unwrap()).unwrap();

        // 2. Initialize UserRepo
        let repo = UserRepo::new(SafePath::from_trusted_root_path(root.clone()));

        // 3. Trigger migration via get_layouts
        let corne_layouts = repo.get_layouts("corne");

        // 4. Assert data is present
        assert_eq!(corne_layouts.len(), 1);
        assert_eq!(
            corne_layouts
                .get("My Layout")
                .map(std::string::String::as_str),
            Some("Q W E R T Y U I O P")
        );

        // 5. Assert migration happened
        assert!(!legacy_path.exists(), "Legacy file should be moved");
        assert!(
            root.join("user/user_layouts.json.bak").exists(),
            "Backup file should exist"
        );

        let corne_file = root.join("user/layouts/corne/My_Layout.json");
        assert!(corne_file.exists(), "New layout file should exist");

        let ansi_file = root.join("user/layouts/ansi/QWERTY.json");
        assert!(ansi_file.exists(), "New layout file should exist");

        // 6. Verify file content
        let content = fs::read_to_string(corne_file).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(json["name"], "My Layout");
        assert_eq!(json["layout"], "Q W E R T Y U I O P");
    }
}
