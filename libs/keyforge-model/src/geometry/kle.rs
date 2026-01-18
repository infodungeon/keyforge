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
    
    // Pass 1: Collect X coordinates for clustering
    let mut x_coords: Vec<f32> = keyboard.keys.iter().map(|k| k.x as f32 + (k.width as f32 / 2.0)).collect();
    x_coords.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    
    // Determine split point using largest gap if enough keys exist
    let split_x = if x_coords.len() > 2 {
        let mut max_gap = 0.0;
        let mut split = 10.0; // Fallback
        for i in 0..x_coords.len()-1 {
            let gap = x_coords[i+1] - x_coords[i];
            if gap > max_gap {
                max_gap = gap;
                split = x_coords[i] + (gap / 2.0);
            }
        }
        // Heuristic: If max gap is small (ortho/compact), fall back to median
        if max_gap < 1.5 {
            x_coords[x_coords.len() / 2]
        } else {
            split
        }
    } else {
        10.0
    };

    let mut keys = Vec::new();

    for (current_id, key) in keyboard.keys.into_iter().enumerate() {
        let center_x = key.x + (key.width / 2.0);
        // Dynamic hand assignment
        let hand = if center_x as f32 > split_x { HandIndex::RIGHT } else { HandIndex::LEFT };
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
