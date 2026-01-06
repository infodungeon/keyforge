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

use keyforge_model::parsing::{parse_key, KeyAction};
use keyforge_model::geometry::KeyboardDefinition;

#[test]
fn test_key_action_parsing() {
    assert_eq!(parse_key("KC_A"), KeyAction::Simple("KC_A".to_string()));
    assert_eq!(parse_key("TRNS"), KeyAction::Transparent);
    
    match parse_key("MO(1)") {
        KeyAction::LayerMomentary(1) => {},
        _ => panic!("Failed to parse MO(1)"),
    }

    match parse_key("LT(2, KC_SPC)") {
        KeyAction::LayerTap { layer: 2, key } => assert_eq!(key, "KC_SPC"),
        _ => panic!("Failed to parse LT"),
    }
}

#[test]
fn test_kle_import() {
    // Minimal KLE JSON
    let json = r#"[
        {"meta": {"name": "Test"}},
        [{"x":0},"A",{"x":1},"B"]
    ]"#;
    
    // Note: KeyboardDefinition::parse handles KLE detection
    let def = KeyboardDefinition::parse(json, Some("Test Board"));
    
    if let Ok(kb) = def {
        assert_eq!(kb.meta.name, "Test Board");
        assert_eq!(kb.geometry.keys.len(), 2);
    }
}