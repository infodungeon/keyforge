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
use keyforge_infra::config::CommonConfig;
use keyforge_infra::{sanitize_filename, parse_layout_string_permissive_cached};
use keyforge_model::keycodes::KeycodeRegistry;

#[test]
fn test_config_from_env() {
    temp_env::with_var("KEYFORGE_HIVE_URL", Some("http://test.local"), || {
        let cfg = CommonConfig::from_env();
        assert_eq!(cfg.hive_url.unwrap(), "http://test.local");
    });
}

#[test]
fn test_filename_sanitization() {
    // The allowlist includes '.', '-', and '_'. Slashes are replaced by '_'.
    // "../../etc/passwd" -> ".." + "_" + ".." + "_" + "etc" + "_" + "passwd"
    assert_eq!(sanitize_filename("../../etc/passwd"), ".._.._etc_passwd");
    assert_eq!(sanitize_filename("valid-file.json"), "valid-file.json");
}

#[test]
fn test_layout_parser() {
    let registry = KeycodeRegistry::default(); 
    
    let layout_str = "KC_A KC_B";
    let layout = parse_layout_string_permissive_cached(layout_str, 2, &registry).unwrap();
    
    assert_eq!(layout.keys.len(), 2);
}