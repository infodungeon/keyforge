// libs/keyforge-model/src/geometry/kle.rs

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


//! Keyboard Layout Editor (KLE) integration.
//!
//! This module provides functions for importing and exporting 
//! keyboard geometries in the KLE JSON format.

use super::{KeyNode, KeyboardGeometry};
use crate::types::{ColIndex, FingerIndex, HandIndex, KeyIndex, RowIndex};
use kle_serial::Keyboard as KleKeyboard;
use serde_json::json;
use std::error::Error;

/// Parses a Keyboard Layout Editor (KLE) JSON string into a `KeyboardGeometry`.
pub fn parse_kle_json(content: &str) -> Result<KeyboardGeometry, Box<dyn Error>> {
    let keyboard: KleKeyboard = serde_json::from_str(content)?;
    let mut keys = Vec::new();

    for (current_id, key) in keyboard.keys.into_iter().enumerate() {
        let center_x = key.x + (key.width / 2.0);
        let hand = if center_x > 10.0 { HandIndex::RIGHT } else { HandIndex::LEFT };
        let finger = FingerIndex::INDEX;

        let label = key.legends.iter().flatten().find(|l| !l.text.is_empty())
            .map(|l| l.text.as_str()).unwrap_or("").to_string();

        let node = KeyNode {
            index: current_id,
            label: if label.is_empty() { format!("k{}", current_id) } else { label },
            hand,
            finger,
            row: RowIndex(key.y.round() as i8),
            col: ColIndex(key.x.round() as i8),
            x: key.x as f32,
            y: key.y as f32,
            w: key.width as f32,
            h: key.height as f32,
            r: key.rotation as f32,
            rx: key.rx as f32,
            ry: key.ry as f32,
            is_home: false,
            is_stretch: false,
        };
        keys.push(node);
    }

    let total = keys.len();
    let prime_slots = (0..std::cmp::min(8, total)).map(|i| KeyIndex(i as u16)).collect();
    let med_slots = (8..std::cmp::min(20, total)).map(|i| KeyIndex(i as u16)).collect();
    let low_slots = (20..total).map(|i| KeyIndex(i as u16)).collect();

    let geom = KeyboardGeometry {
        keys,
        prime_slots,
        med_slots,
        low_slots,
        home_row: 1,
    };
    Ok(geom)
}

/// Converts a `KeyboardGeometry` back into a KLE JSON string.
pub fn to_kle_json(geom: &KeyboardGeometry) -> Result<String, Box<dyn Error>> {
    let mut json_rows = Vec::new();
    json_rows.push(json!({ "meta": { "name": "KeyForge Export", "author": "KeyForge" } }));

    for k in &geom.keys {
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
