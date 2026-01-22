// libs/keyforge-adapter/src/parsing.rs

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

//! Protocol parsing facade.
//!
//! This module provides utilities for parsing external keymap formats
//! (like QMK or ZMK) into internal domain representations.

use crate::error::{AdapterError, AdapterResult};
use serde::{Deserialize, Serialize};
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

/// A parsable key action from an external format.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub enum KeyAction {
    /// A simple keycode (e.g., "KC_A").
    Simple(String),
    /// Transparent (pass-through).
    Transparent,
    /// No Operation.
    NoOp,
    /// Momentary layer switch (MO).
    LayerMomentary(u8),
    /// Toggle layer (TG).
    LayerToggle(u8),
    /// Turn on layer (TO).
    LayerOn(u8),
    /// Modifier Tap (Hold for Mod, Tap for Key).
    ModTap {
        /// The modifier (e.g., "LSHIFT").
        mod_name: String,
        /// The tap key (recursive).
        key: Box<KeyAction>,
    },
    /// Layer Tap (Hold for Layer, Tap for Key).
    LayerTap {
        /// The layer index.
        layer: u8,
        /// The tap key (recursive).
        key: Box<KeyAction>,
    },
    /// Sticky Modifier (One-Shot Mod).
    StickyMod(String),
    /// Caps Word behavior.
    CapsWord,
}

/// Parses a string token into a `KeyAction` using a recursive descent approach.
///
/// # Errors
///
/// Returns `AdapterError::Validation` if the token is malformed or exceeds safety limits.
pub fn parse_key(token: &str) -> AdapterResult<KeyAction> {
    let t = token.trim();
    if t.len() > 100 {
        // Safety check for recursion depth/exploit
        return Err(AdapterError::Validation(format!("Token too long: {t}")));
    }
    let upper = t.to_uppercase();

    // 1. Simple Keywords
    match upper.as_str() {
        "TRNS" | keyforge_model::constants::DEFAULT_TRANSPARENT | "_" => {
            return Ok(KeyAction::Transparent)
        }
        "NO" | keyforge_model::constants::DEFAULT_NO_OP | "XXX" => return Ok(KeyAction::NoOp),
        "CAPS_WORD" | "CW" => return Ok(KeyAction::CapsWord),
        _ => {}
    }

    // 2. Function Call Parsing (NAME(ARG, ...))
    if let Some((name, args_str)) = parse_function_call(&upper) {
        match name {
            "MO" => {
                if let Ok(l) = args_str.parse::<u8>() {
                    return Ok(KeyAction::LayerMomentary(l));
                }
            }
            "TG" => {
                if let Ok(l) = args_str.parse::<u8>() {
                    return Ok(KeyAction::LayerToggle(l));
                }
            }
            "TO" => {
                if let Ok(l) = args_str.parse::<u8>() {
                    return Ok(KeyAction::LayerOn(l));
                }
            }
            "LT" => {
                if let Some((layer_str, key_str)) = split_args_safe(&args_str) {
                    if let Ok(layer) = layer_str.trim().parse::<u8>() {
                        // RECURSIVE CALL
                        let key_action = parse_key(&key_str)?;
                        return Ok(KeyAction::LayerTap {
                            layer,
                            key: Box::new(key_action),
                        });
                    }
                }
            }
            "MT" => {
                if let Some((mod_str, key_str)) = split_args_safe(&args_str) {
                    let key_action = parse_key(&key_str)?;
                    return Ok(KeyAction::ModTap {
                        mod_name: mod_str.trim().to_string(),
                        key: Box::new(key_action),
                    });
                }
            }
            "SK" | "OSM" => {
                return Ok(KeyAction::StickyMod(args_str.trim().to_string()));
            }
            _ if name.ends_with("_T") => {
                // MOD_T(KEY) or LSHIFT_T(KEY)
                // Extract mod name from function name
                let mod_name = name.trim_end_matches("_T").to_string();
                let key_action = parse_key(&args_str)?;
                return Ok(KeyAction::ModTap {
                    mod_name,
                    key: Box::new(key_action),
                });
            }
            _ => {
                return Err(AdapterError::Validation(format!(
                    "Unknown function call: {name}"
                )));
            }
        }
    }

    // 3. Fallback to Simple
    if t.contains('(') || t.contains(')') {
        return Err(AdapterError::Validation(format!(
            "Malformed function call: {t}"
        )));
    }

    // Normalize simple keys
    if !upper.starts_with("KC_") && upper.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Ok(KeyAction::Simple(format!("KC_{upper}")));
    }

    Ok(KeyAction::Simple(t.to_string()))
}

/// Helper: Extracts `NAME` and `ARGS` from `NAME(ARGS)`.
fn parse_function_call(s: &str) -> Option<(&str, String)> {
    if let Some(idx) = s.find('(') {
        if s.ends_with(')') {
            let name = &s[..idx];
            let args = &s[idx + 1..s.len() - 1];
            return Some((name.trim(), args.to_string()));
        }
    }
    None
}

/// Helper: Splits arguments by comma, respecting nested parentheses.
fn split_args_safe(s: &str) -> Option<(String, String)> {
    let mut depth = 0;
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                let first = s[..i].to_string();
                let second = s[i + 1..].to_string();
                return Some((first, second));
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_action_parsing() {
        assert_eq!(
            parse_key("KC_A").unwrap(),
            KeyAction::Simple("KC_A".to_string())
        );
        assert_eq!(parse_key("TRNS").unwrap(), KeyAction::Transparent);

        match parse_key("MO(1)").unwrap() {
            KeyAction::LayerMomentary(1) => {}
            _ => panic!("Failed to parse MO(1)"),
        }

        match parse_key("LT(2, KC_SPC)").unwrap() {
            KeyAction::LayerTap { layer: 2, key } => {
                assert_eq!(*key, KeyAction::Simple("KC_SPC".to_string()));
            }
            _ => panic!("Failed to parse LT"),
        }
    }
}