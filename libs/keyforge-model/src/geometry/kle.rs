// libs/keyforge-model/src/geometry/kle.rs

use crate::geometry::{KeyNode, KeyboardGeometry};
use crate::types::{HandIndex, RowIndex, SpatialUnit, FingerIndex, ColIndex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Kle format structures for parsing
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct KleKeyProps {
    #[serde(default)]
    x: f32,
    #[serde(default)]
    y: f32,
    #[serde(default)]
    w: f32,
    #[serde(default)]
    h: f32,
    #[serde(default)]
    r: f32,
    #[serde(default)]
    rx: f32,
    #[serde(default)]
    ry: f32,
}

/// Parses a KLE JSON array into a `KeyboardGeometry`.
pub fn parse_kle_json(json: &str) -> Result<KeyboardGeometry, String> {
    let raw: Vec<serde_json::Value> = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let mut keys = Vec::new();
    let mut current_x = 0.0;
    let mut current_y = 0.0;
    let mut current_props = KleKeyProps::default();

    for row in raw {
        if let Some(row_array) = row.as_array() {
            for item in row_array {
                if let Some(props) = item.as_object() {
                    // Update current properties
                    if let Some(x) = props.get("x").and_then(|v| v.as_f64()) {
                        current_x += x as f32;
                    }
                    if let Some(y) = props.get("y").and_then(|v| v.as_f64()) {
                        current_y += y as f32;
                    }
                    if let Some(r) = props.get("r").and_then(|v| v.as_f64()) {
                        current_props.r = r as f32;
                    }
                    if let Some(rx) = props.get("rx").and_then(|v| v.as_f64()) {
                        current_props.rx = rx as f32;
                    }
                    if let Some(ry) = props.get("ry").and_then(|v| v.as_f64()) {
                        current_props.ry = ry as f32;
                    }
                } else if let Some(label) = item.as_str() {
                    // It's a key
                    let hand = if current_x < 5.0 { HandIndex::LEFT } else { HandIndex::RIGHT };
                    
                    keys.push(KeyNode {
                        index: keys.len(),
                        label: label.to_string(),
                        x: SpatialUnit::from_f32(current_x),
                        y: SpatialUnit::from_f32(current_y),
                        w: 1.0,
                        h: 1.0,
                        r: current_props.r,
                        rx: SpatialUnit::from_f32(current_props.rx),
                        ry: SpatialUnit::from_f32(current_props.ry),
                        hand,
                        finger: FingerIndex::new_unchecked(0),
                        row: RowIndex::new(current_y as i8),
                        col: ColIndex::new(current_x as i8),
                        ..Default::default()
                    });
                    current_x += 1.0;
                }
            }
            current_x = 0.0;
            current_y += 1.0;
        }
    }

    Ok(KeyboardGeometry {
        keys,
        prime_slots: Vec::new(),
        med_slots: Vec::new(),
        low_slots: Vec::new(),
        home_row: RowIndex::new(1),
    })
}

/// Simple placeholder for KLE export
pub fn to_kle_json(_geo: &KeyboardGeometry) -> Result<String, String> {
    Ok("{\"meta\": {}}".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SpatialUnit;

    #[test]
    fn test_kle_basic_parsing() {
        let json = r#"[["A", "B", {"x": 1}, "C"]]"#;
        let geom = parse_kle_json(json).unwrap();
        assert_eq!(geom.keys.len(), 3);
        assert_eq!(geom.keys[0].hand, HandIndex::LEFT);
        assert_eq!(geom.keys[2].hand, HandIndex::RIGHT);
    }

    #[test]
    fn test_kle_rotation_parsing() {
        let json = r#"[
            [{"r": 15, "rx": 5, "ry": 5}, "A"]
        ]"#;
        let geom = parse_kle_json(json).unwrap();
        assert_eq!(geom.keys[0].r, 15.0);
        assert_eq!(geom.keys[0].rx, SpatialUnit::from_f32(5.0));
        assert_eq!(geom.keys[0].ry, SpatialUnit::from_f32(5.0));
    }
}