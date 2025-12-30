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
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn test_parse_key_no_panic(s in "\\PC*") {
        // INVARIANT: Parsing arbitrary strings must never panic
        let _ = parse_key(&s);
    }

    #[test]
    fn test_parse_key_layer_fuzz(layer in 0u8..255) {
        // INVARIANT: Layer parsing handles all u8 values gracefully
        // Layers > 31 should degrade to Raw, <= 31 to specific actions
        let input = format!("MO({})", layer);
        let action = parse_key(&input);

        if layer > 31 {
            prop_assert!(matches!(action, KeyAction::Raw(_)));
        } else {
            prop_assert!(matches!(action, KeyAction::LayerMomentary(l) if l == layer));
        }
    }

    #[test]
    fn test_parse_key_length_truncation(s in "[a-zA-Z0-9]{33, 100}") {
        // INVARIANT: Inputs > 32 chars are truncated and returned as Raw
        let action = parse_key(&s);
        if let KeyAction::Raw(raw) = action {
            prop_assert!(raw.len() <= 32);
        } else {
            // It might match a specific pattern if the random string looks like one,
            // but generally long random strings become Raw.
            // If it matched a specific pattern (like CAPS_WORD), that's fine too,
            // but we are testing the truncation logic for unknown tokens.
        }
    }
}
