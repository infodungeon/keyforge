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
use keyforge_protocol::parsing::{parse_key, KeyAction};

#[test]
fn test_parse_simple() {
    assert_eq!(parse_key("KC_A"), KeyAction::Simple("KC_A".into()));
    assert_eq!(parse_key("A"), KeyAction::Simple("KC_A".into()));
    assert_eq!(parse_key("1"), KeyAction::Simple("KC_1".into()));
}

#[test]
fn test_parse_special_constants() {
    assert_eq!(parse_key("TRNS"), KeyAction::Transparent);
    assert_eq!(parse_key("_______"), KeyAction::Transparent);
    assert_eq!(parse_key("NO"), KeyAction::NoOp);
    assert_eq!(parse_key("XXXXXXX"), KeyAction::NoOp);
    assert_eq!(parse_key("CAPS_WORD"), KeyAction::CapsWord);
    assert_eq!(parse_key("CW"), KeyAction::CapsWord);
}

#[test]
fn test_parse_layers() {
    assert_eq!(parse_key("MO(1)"), KeyAction::LayerMomentary(1));
    assert_eq!(parse_key("TG(2)"), KeyAction::LayerToggle(2));
    assert_eq!(parse_key("TO(3)"), KeyAction::LayerOn(3));

    // Invalid layer index (too high) falls back to Raw
    assert!(matches!(parse_key("MO(255)"), KeyAction::Raw(_)));
}

#[test]
fn test_parse_mod_tap() {
    match parse_key("LSFT_T(KC_A)") {
        KeyAction::ModTap { mod_name, key } => {
            assert_eq!(mod_name, "LSFT");
            assert_eq!(key, "KC_A");
        }
        _ => panic!("Expected ModTap"),
    }
}

#[test]
fn test_parse_layer_tap() {
    match parse_key("LT(1, KC_ENT)") {
        KeyAction::LayerTap { layer, key } => {
            assert_eq!(layer, 1);
            assert_eq!(key, "KC_ENT");
        }
        _ => panic!("Expected LayerTap"),
    }
}

#[test]
fn test_parse_sticky_mod() {
    assert_eq!(
        parse_key("SK(KC_LSFT)"),
        KeyAction::StickyMod("KC_LSFT".into())
    );
    assert_eq!(
        parse_key("OSM(MOD_LCTL)"),
        KeyAction::StickyMod("MOD_LCTL".into())
    );
}

#[test]
fn test_parse_raw_fallback() {
    // Unknown format
    assert!(matches!(parse_key("UNKNOWN(1)"), KeyAction::Raw(_)));
    // Too long
    let long = "A".repeat(50);
    assert!(matches!(parse_key(&long), KeyAction::Raw(_)));
}
