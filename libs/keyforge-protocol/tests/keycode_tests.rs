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
use keyforge_protocol::keycodes::{KeycodeDefinition, KeycodeRegistry};
use keyforge_protocol::types::KeyCode;

#[test]
fn test_registry_lookup() {
    let defs = vec![KeycodeDefinition {
        code: KeyCode(65),
        id: "KC_A".into(),
        label: "A".into(),
        aliases: vec!["A".into()],
    }];
    let reg = KeycodeRegistry::new(defs);

    assert_eq!(reg.get_code("KC_A"), Some(KeyCode(65)));
    assert_eq!(reg.get_code("A"), Some(KeyCode(65)));
    assert_eq!(reg.get_code("a"), Some(KeyCode(65))); // Case insensitive
    assert_eq!(reg.get_code("B"), None);

    assert_eq!(reg.get_label(KeyCode(65)), "A");
    assert_eq!(reg.get_label(KeyCode(66)), "[66]"); // Fallback
}

#[test]
fn test_registry_defaults() {
    let reg = KeycodeRegistry::new_with_defaults();
    assert!(reg.get_code("KC_NO").is_some());
    assert!(reg.get_code("KC_TRNS").is_some());
}
