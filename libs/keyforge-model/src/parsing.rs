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

use regex::Regex;
use std::sync::OnceLock;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

/// Represents a parsed action from a keymap file (e.g., QMK/ZMK format).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub enum KeyAction {
    /// A simple keycode (e.g., "KC_A").
    Simple(String),
    /// Transparent (pass-through to lower layer).
    Transparent,
    /// No Operation (does nothing).
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
        /// The tap key (e.g., "KC_A").
        key: String 
    },
    /// Layer Tap (Hold for Layer, Tap for Key).
    LayerTap { 
        /// The layer index.
        layer: u8, 
        /// The tap key.
        key: String 
    },
    /// Sticky Modifier (One-Shot Mod).
    StickyMod(String),
    /// Caps Word behavior.
    CapsWord,
    /// Unparsed raw string.
    Raw(String),
}

static MOD_TAP_RE: OnceLock<Regex> = OnceLock::new();
static LAYER_TAP_RE: OnceLock<Regex> = OnceLock::new();
static LAYER_ACTION_RE: OnceLock<Regex> = OnceLock::new();
static STICKY_MOD_RE: OnceLock<Regex> = OnceLock::new();

const MAX_LAYER: u8 = 31;
const MAX_TOKEN_LEN: usize = 32;

/// Parses a string token into a `KeyAction`.
pub fn parse_key(token: &str) -> KeyAction {
    let t = token.trim();
    if t.len() > MAX_TOKEN_LEN {
        return KeyAction::Raw(t.chars().take(MAX_TOKEN_LEN).collect());
    }
    let upper = t.to_uppercase();

    if matches!(upper.as_str(), "TRNS" | "_______" | "_") { return KeyAction::Transparent; }
    if matches!(upper.as_str(), "NO" | "XXXXXXX" | "XXX") { return KeyAction::NoOp; }
    if matches!(upper.as_str(), "CAPS_WORD" | "CW") { return KeyAction::CapsWord; }

    let layer_re = LAYER_ACTION_RE.get_or_init(|| Regex::new(r"^(MO|TG|TO)\((\d+)\)$").expect("static regex"));
    if let Some(caps) = layer_re.captures(&upper) {
        if let (Some(action_match), Some(layer_match)) = (caps.get(1), caps.get(2)) {
            let action = action_match.as_str();
            let layer = layer_match.as_str().parse::<u8>().unwrap_or(0);
            if layer > MAX_LAYER { return KeyAction::Raw(t.to_string()); }
            return match action {
                "MO" => KeyAction::LayerMomentary(layer),
                "TG" => KeyAction::LayerToggle(layer),
                "TO" => KeyAction::LayerOn(layer),
                _ => KeyAction::Raw(t.to_string()),
            };
        }
    }

    let lt_re = LAYER_TAP_RE.get_or_init(|| Regex::new(r"^LT\((\d+),\s*(.+)\)$").expect("static regex"));
    if let Some(caps) = lt_re.captures(&upper) {
        if let (Some(layer_match), Some(key_match)) = (caps.get(1), caps.get(2)) {
            let layer = layer_match.as_str().parse::<u8>().unwrap_or(0);
            let key = key_match.as_str().trim().to_string();
            if layer > MAX_LAYER { return KeyAction::Raw(t.to_string()); }
            return KeyAction::LayerTap { layer, key };
        }
    }

    let mt_re = MOD_TAP_RE.get_or_init(|| Regex::new(r"^([A-Z0-9_]+)_T\((.+)\)$").expect("static regex"));
    if let Some(caps) = mt_re.captures(&upper) {
        if let (Some(mod_match), Some(key_match)) = (caps.get(1), caps.get(2)) {
            let mod_name = mod_match.as_str().to_string();
            let key = key_match.as_str().trim().to_string();
            return KeyAction::ModTap { mod_name, key };
        }
    }

    let sk_re = STICKY_MOD_RE.get_or_init(|| Regex::new(r"^(?:SK|OSM)\((.+)\)$").expect("static regex"));
    if let Some(caps) = sk_re.captures(&upper) {
        if let Some(mod_match) = caps.get(1) {
            return KeyAction::StickyMod(mod_match.as_str().to_string());
        }
    }

    if t.contains('(') || t.contains(')') { return KeyAction::Raw(t.to_string()); }
    if t.len() == 1 && t.chars().next().is_some_and(|c| c.is_alphanumeric()) {
        return KeyAction::Simple(format!("KC_{}", upper));
    }
    KeyAction::Simple(t.to_string())
}
