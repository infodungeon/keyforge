// crates/keyforge-protocol/src/geometry/kle.rs
use super::{KeyNode, KeyboardGeometry};
use kle_serial::Keyboard as KleKeyboard;
use serde_json::json;
use std::error::Error;

pub fn parse_kle_json(content: &str) -> Result<KeyboardGeometry, Box<dyn Error>> {
    let keyboard: KleKeyboard = serde_json::from_str(content)?;

    let mut keys = Vec::new();

    for (current_id, key) in keyboard.keys.into_iter().enumerate() {
        let center_x = key.x + (key.width / 2.0);
        // Heuristic for split boards: if x > 10, assume right hand.
        let hand = if center_x > 10.0 { 1 } else { 0 };
        let finger = 1; // Default to index

        // Extract label (first legend)
        let label = key
            .legends
            .iter()
            .flatten()
            .find(|l| !l.text.is_empty())
            .map(|l| l.text.as_str())
            .unwrap_or("")
            .to_string();

        let node = KeyNode {
            id: if label.is_empty() {
                format!("k{}", current_id)
            } else {
                label
            },
            hand,
            finger,
            row: key.y.round() as i8,
            col: key.x.round() as i8,
            x: key.x as f32,
            y: key.y as f32,
            w: key.width as f32,
            h: key.height as f32,
            r: key.rotation as f32,
            rx: key.rx as f32,
            ry: key.ry as f32,
            is_stretch: false,
        };

        keys.push(node);
    }

    // Auto-assign slots based on count
    let total = keys.len();
    let prime_slots = (0..std::cmp::min(8, total)).collect();
    let med_slots = (8..std::cmp::min(20, total)).collect();
    let low_slots = (20..total).collect();

    let geom = KeyboardGeometry {
        keys,
        prime_slots,
        med_slots,
        low_slots,
        home_row: 1,
    };

    Ok(geom)
}

pub fn to_kle_json(geom: &KeyboardGeometry) -> Result<String, Box<dyn Error>> {
    let mut json_rows = Vec::new();

    // 1. Metadata
    json_rows.push(json!({
        "meta": {
            "name": "KeyForge Export",
            "author": "KeyForge"
        }
    }));

    // 2. Keys
    // To ensure absolute positioning works in KLE, we treat every key as a new row
    // with absolute x/y coordinates.
    for k in &geom.keys {
        let props = json!({
            "x": k.x,
            "y": k.y,
            "w": k.w,
            "h": k.h,
            "r": k.r,
            "rx": k.rx,
            "ry": k.ry,
            // Visual styling
            "c": if k.hand == 0 { "#cccccc" } else { "#aaaaaa" },
            "a": 7 // Center alignment
        });

        // The row format in KLE is [ {props}, "Label" ]
        let row = json!([props, k.id]);
        json_rows.push(row);
    }

    let json_str = serde_json::to_string_pretty(&json_rows)?;
    Ok(json_str)
}
