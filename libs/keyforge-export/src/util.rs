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

//! Shared utilities for keymap exporters.

/// Formats for modifier name translation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModFormat {
    /// QMK-style modifiers (e.g. `MOD_LSFT`).
    Qmk,
    /// ZMK-style modifiers (e.g. LSHIFT).
    Zmk,
}

/// Maps a modifier name to a firmware-specific representation.
#[must_use]
pub fn map_modifier(name: &str, format: ModFormat) -> String {
    let upper = name.to_uppercase();
    let base = match upper.as_str() {
        "LSFT" | "SFT" | "LSHIFT" => "LSFT",
        "RSFT" | "RSHIFT" => "RSFT",
        "LCTL" | "CTL" | "LCTRL" => "LCTL",
        "RCTL" | "RCTRL" => "RCTL",
        "LALT" | "ALT" => "LALT",
        "RALT" | "ALGR" => "RALT",
        "LGUI" | "GUI" | "WIN" | "CMD" => "LGUI",
        "RGUI" => "RGUI",
        _ => return name.to_string(),
    };

    match format {
        ModFormat::Qmk => match base {
            "LSFT" => "MOD_LSFT".to_string(),
            "RSFT" => "MOD_RSFT".to_string(),
            "LCTL" => "MOD_LCTL".to_string(),
            "RCTL" => "MOD_RCTL".to_string(),
            "LALT" => "MOD_LALT".to_string(),
            "RALT" => "MOD_RALT".to_string(),
            "LGUI" => "MOD_LGUI".to_string(),
            "RGUI" => "MOD_RGUI".to_string(),
            _ => base.to_string(),
        },
        ModFormat::Zmk => match base {
            "LSFT" => "LSHIFT".to_string(),
            "RSFT" => "RSHIFT".to_string(),
            "LCTL" => "LCTRL".to_string(),
            "RCTL" => "RCTRL".to_string(),
            "LALT" => "LALT".to_string(),
            "RALT" => "RALT".to_string(),
            "LGUI" => "LGUI".to_string(),
            "RGUI" => "RGUI".to_string(),
            _ => base.to_string(),
        },
    }
}

/// Sanitizes a string for use as a C identifier.
#[must_use]
pub fn sanitize_c(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

/// Sanitizes a string for use in a ZMK devicetree (more restrictive).
#[must_use]
pub fn sanitize_zmk(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_modifier_qmk() {
        assert_eq!(map_modifier("LSFT", ModFormat::Qmk), "MOD_LSFT");
        assert_eq!(map_modifier("LSHIFT", ModFormat::Qmk), "MOD_LSFT");
        assert_eq!(map_modifier("WIN", ModFormat::Qmk), "MOD_LGUI");
        assert_eq!(map_modifier("XYZ", ModFormat::Qmk), "XYZ");
    }

    #[test]
    fn test_map_modifier_zmk() {
        assert_eq!(map_modifier("LSFT", ModFormat::Zmk), "LSHIFT");
        assert_eq!(map_modifier("RSHIFT", ModFormat::Zmk), "RSHIFT");
        assert_eq!(map_modifier("GUI", ModFormat::Zmk), "LGUI");
    }

    #[test]
    fn test_sanitize_c() {
        assert_eq!(sanitize_c("Layout Name"), "Layout_Name");
        assert_eq!(sanitize_c("Key(1)"), "Key_1");
        assert_eq!(sanitize_c("__Extra_"), "Extra");
    }

    #[test]
    fn test_sanitize_zmk() {
        assert_eq!(sanitize_zmk("Layout Name"), "LayoutName");
        assert_eq!(sanitize_zmk("Key(1)"), "Key1");
        assert_eq!(sanitize_zmk("my_id"), "my_id");
    }
}
