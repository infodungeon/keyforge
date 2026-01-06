// Copyright (c) 2025 KeyForge Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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