// libs/keyforge-model/src/geometry/kle.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You    may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Keyboard Layout Editor (KLE) integration.
//!
//! This module provides functions for importing and exporting
//! keyboard geometries in the KLE JSON format.

use super::{KeyNode, KeyboardGeometry};
use crate::types::{ColIndex, FingerIndex, HandIndex, KeyIndex, RowIndex, SpatialUnit};
use kle_serial::Keyboard as KleKeyboard;
use regex::Regex;
use serde_json::json;
use std::error::Error;
use std::sync::OnceLock;

static LABEL_CLEANER: OnceLock<Regex> = OnceLock::new();

/// Parses a Keyboard Layout Editor (KLE) JSON string into a `KeyboardGeometry`.
///
/// # Errors
///
/// Returns an error if the JSON is malformed or doesn't match the expected schema.
#[allow(clippy::too_many_lines)]
pub fn parse_kle_json(content: &str) -> Result<KeyboardGeometry, Box<dyn Error>> {
    let keyboard: KleKeyboard = serde_json::from_str(content)?;

    // Pass 1: Collect X coordinates for clustering
    let mut x_coords: Vec<f32> = keyboard
        .keys
        .iter()
        .map(|k| {
            #[allow(clippy::cast_possible_truncation)]
            let x_coord = k.x as f32;
            #[allow(clippy::cast_possible_truncation)]
            let width_val = k.width as f32;
            x_coord + (width_val / 2.0)
        })
        .collect();
    x_coords.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    // Determine split point using largest gap closest to center
    let split_x = if x_coords.len() > 2 {
        let center = f32::midpoint(x_coords[0], x_coords[x_coords.len() - 1]);
        let mut best_split = center;
        let mut min_dist_to_center = f32::MAX;
        let mut max_gap = 0.0;

        for i in 0..x_coords.len() - 1 {
            let gap = x_coords[i + 1] - x_coords[i];
            let split = x_coords[i] + (gap / 2.0);
            let dist = (split - center).abs();

            // Heuristic: Prefer large gaps near the center
            if gap > 1.2 {
                if dist < min_dist_to_center {
                    min_dist_to_center = dist;
                    best_split = split;
                    max_gap = gap;
                }
            } else if gap > max_gap {
                // If no large gaps, take the absolute largest
                max_gap = gap;
                best_split = split;
            }
        }

        if max_gap < 0.5 {
            x_coords[x_coords.len() / 2]
        } else {
            best_split
        }
    } else {
        10.0
    };

    let mut keys = Vec::new();

    for (current_id, key) in keyboard.keys.into_iter().enumerate() {
        #[allow(clippy::cast_possible_truncation)]
        let center_x_raw = (key.x + (key.width / 2.0)) as f32;
        let center_x = center_x_raw;
        // Dynamic hand assignment
        let hand = if center_x > split_x {
            HandIndex::RIGHT
        } else {
            HandIndex::LEFT
        };
        let finger = FingerIndex::INDEX;

        let label = key
            .legends
            .iter()
            .flatten()
            .find(|l| !l.text.is_empty())
            .map_or("", |l| l.text.as_str());

        let label = sanitize_label(label);

        #[allow(clippy::cast_possible_truncation)]
        let key_x = key.x as f32;
        #[allow(clippy::cast_possible_truncation)]
        let key_y = key.y as f32;
        #[allow(clippy::cast_possible_truncation)]
        let key_w = key.width as f32;
        #[allow(clippy::cast_possible_truncation)]
        let key_h = key.height as f32;
        #[allow(clippy::cast_possible_truncation)]
        let key_r = key.rotation as f32;
        #[allow(clippy::cast_possible_truncation)]
        let key_rx_val = key.rx as f32;
        #[allow(clippy::cast_possible_truncation)]
        let key_ry_coord = key.ry as f32;

        let node = KeyNode {
            index: KeyIndex::new(u16::try_from(current_id).unwrap_or(u16::MAX)),
            label: if label.is_empty() {
                format!("k{current_id}")
            } else {
                label
            },
            hand,
            finger,
            row: RowIndex::new(
                i8::try_from(
                    crate::types::Score::from_f32(key_y)
                        .unwrap_or_default()
                        .raw()
                        / 1_000_000,
                )
                .unwrap_or(0),
            ),
            col: ColIndex::new(
                i8::try_from(
                    crate::types::Score::from_f32(key_x)
                        .unwrap_or_default()
                        .raw()
                        / 1_000_000,
                )
                .unwrap_or(0),
            ),
            x: SpatialUnit::from_f32(key_x),
            y: SpatialUnit::from_f32(key_y),
            w: key_w,
            h: key_h,
            r: key_r,
            rx: SpatialUnit::from_f32(key_rx_val),
            ry: SpatialUnit::from_f32(key_ry_coord),
            is_home: false,
            is_stretch: false,
        };
        keys.push(node);
    }

    let total = keys.len();
    let prime_slots = (0..std::cmp::min(8, total))
        .map(|i| KeyIndex::new(u16::try_from(i).unwrap_or(0)))
        .collect();
    let med_slots = (8..std::cmp::min(20, total))
        .map(|i| KeyIndex::new(u16::try_from(i).unwrap_or(0)))
        .collect();
    let low_slots = (20..total)
        .map(|i| KeyIndex::new(u16::try_from(i).unwrap_or(0)))
        .collect();

    let geom = KeyboardGeometry::new(keys, prime_slots, med_slots, low_slots, RowIndex::new(1));
    Ok(geom)
}

/// Sanitizes a KLE label by stripping HTML tags and common escape sequences.
fn sanitize_label(label: &str) -> String {
    let cleaner = LABEL_CLEANER.get_or_init(|| {
        // Safe regex to strip common HTML tags found in KLE (<i>, <b>, <br>, etc)
        #[allow(clippy::expect_used)]
        Regex::new(r"<[^>]*>").expect("Failed to compile label cleaner regex")
    });
    cleaner.replace_all(label, "").trim().to_string()
}

/// Converts a `KeyboardGeometry` back into a KLE JSON string.
///
/// # Errors
///
/// Returns an error if serialization fails.
pub fn to_kle_json(geom: &KeyboardGeometry) -> Result<String, Box<dyn Error>> {
    let mut json_rows = Vec::new();
    json_rows.push(json!({ "meta": { "name": "KeyForge Export", "author": "KeyForge" } }));

    for k in geom.keys() {
        let props = json!({
            "x": k.x, "y": k.y, "w": k.w, "h": k.h,
            "r": k.r, "rx": k.rx, "ry": k.ry,
            "c": if k.hand == HandIndex::LEFT { "#cccccc" } else { "#aaaaaa" },
            "a": 7
        });
        let row = json!([props, k.label]);
        json_rows.push(row);
    }
    let json_str = serde_json::to_string_pretty(&json_rows)?;
    Ok(json_str)
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;

    #[test]
    fn test_parse_kle_json_simple() -> anyhow::Result<()> {
        let json = r#"[["A", "B"]]"#;
        let geom = parse_kle_json(json).map_err(|e| anyhow::anyhow!(e))?;
        assert_eq!(geom.keys().len(), 2);
        assert_eq!(geom.keys()[0].label, "A");
        assert_eq!(geom.keys()[1].label, "B");
        Ok(())
    }

    #[test]
    fn test_parse_kle_json_split_heuristic() -> anyhow::Result<()> {
        // Large gap (3 keys to hit gap logic)
        let json = r#"[["A", "B", {"x": 15}, "C"]]"#;
        let geom = parse_kle_json(json).map_err(|e| anyhow::anyhow!(e))?;
        assert_eq!(geom.keys()[0].hand, HandIndex::LEFT);
        assert_eq!(geom.keys()[1].hand, HandIndex::LEFT);
        assert_eq!(geom.keys()[2].hand, HandIndex::RIGHT);

        // Small gap (ortho)
        let json = r#"[["A", "B", "C"]]"#;
        let geom = parse_kle_json(json).map_err(|e| anyhow::anyhow!(e))?;
        assert_eq!(geom.keys().len(), 3);
        Ok(())
    }

    #[test]
    fn test_kle_rotation_parsing() -> anyhow::Result<()> {
        let json = r#"[
            [{"r": 15, "rx": 5, "ry": 5}, "A"]
        ]"#;
        let geom = parse_kle_json(json).map_err(|e| anyhow::anyhow!(e))?;
        assert_eq!(geom.keys()[0].r, 15.0);
        assert_eq!(geom.keys()[0].rx.to_f32(), 5.0);
        assert_eq!(geom.keys()[0].ry.to_f32(), 5.0);
        Ok(())
    }

    #[test]
    fn test_to_kle_json() -> anyhow::Result<()> {
        let keys = vec![
            KeyNode {
                label: "X".into(),
                x: SpatialUnit::from_f32(1.0),
                y: SpatialUnit::from_f32(2.0),
                hand: HandIndex::LEFT,
                ..Default::default()
            },
            KeyNode {
                label: "Y".into(),
                x: SpatialUnit::from_f32(10.0),
                y: SpatialUnit::from_f32(2.0),
                hand: HandIndex::RIGHT,
                ..Default::default()
            },
        ];
        let geom = KeyboardGeometry::new(keys, vec![], vec![], vec![], RowIndex::new(1));

        let json = to_kle_json(&geom).map_err(|e| anyhow::anyhow!(e))?;
        assert!(json.contains("meta"));
        assert!(json.contains("\"X\""));
        assert!(json.contains("\"Y\""));
        Ok(())
    }

    #[test]
    fn test_parse_kle_invalid() -> anyhow::Result<()> {
        assert!(parse_kle_json("invalid").is_err());
        Ok(())
    }

    #[test]
    fn test_sanitize_label() -> anyhow::Result<()> {
        assert_eq!(sanitize_label("A"), "A");
        assert_eq!(sanitize_label("<b>A</b>"), "A");
        assert_eq!(sanitize_label("<i class='fa fa-home'></i>"), "");
        assert_eq!(sanitize_label("Shift<br/>Tab"), "ShiftTab");
        assert_eq!(sanitize_label("  Space  "), "Space");
        Ok(())
    }
}
