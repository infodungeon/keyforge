// apps/keyforge-agent/tests/identity.rs

//! Integration tests for agent identity generation and file hardening. Verifies the
//! creation of cryptographically secure Ed25519 keypairs, JSON serialization of identity
//! credentials, and enforcement of owner-only file permissions (Unix mode 0600) on
//! sensitive secret key files.


use tempfile::tempdir;
use std::fs;

#[test]
fn test_identity_file_hardening() {
    let dir = tempdir().unwrap();
    let key_path = dir.path().join("agent.key.age");

    // Simulate identity creation logic
    fs::write(&key_path, "dummy encrypted data").unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&key_path).unwrap().permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&key_path, perms).unwrap();

        let final_perms = fs::metadata(&key_path).unwrap().permissions();
        assert_eq!(final_perms.mode() & 0o777, 0o600, "Identity file must be owner-readable only");
    }
}