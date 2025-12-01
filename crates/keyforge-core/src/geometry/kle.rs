use super::{KeyNode, KeyboardGeometry};
use serde_json::Value;
use std::error::Error;

/// Parses raw KLE JSON content into KeyForge Geometry.
pub fn parse_kle_json(content: &str) -> Result<KeyboardGeometry, Box<dyn Error>> {
    let json: Value = serde_json::from_str(content)?;

    let rows = json.as_array().ok_or("KLE JSON must be an array of rows")?;

    let mut keys = Vec::new();

    // State cursors
    let mut current_y = 0.0;

    for row_val in rows {
        // Skip metadata block (usually the first item is an object defining meta)
        if row_val.is_object() {
            continue;
        }

        let row = row_val.as_array().ok_or("Row must be an array")?;

        let mut current_x = 0.0;
        let mut current_w = 1.0;
        // current_h removed as it was unused in logic

        for item in row {
            if item.is_object() {
                let obj = item.as_object().unwrap();

                // Relative X shift
                if let Some(x) = obj.get("x") {
                    current_x += x.as_f64().unwrap_or(0.0) as f32;
                }

                // Relative Y shift
                if let Some(y) = obj.get("y") {
                    current_y += y.as_f64().unwrap_or(0.0) as f32;
                }

                // Width overrides
                if let Some(w) = obj.get("w") {
                    current_w = w.as_f64().unwrap_or(1.0) as f32;
                }
                // Height 'h' parsing removed (unused)
            } else if item.is_string() {
                let label_full = item.as_str().unwrap();
                // KLE labels can be "Top\nBottom". We usually care about the main label.
                let label = label_full.split('\n').next().unwrap_or("").trim();

                // Default Assignment (User must tweak this later in the UI or JSON)
                // We default to Left Hand (0), Index Finger (1)
                let key = KeyNode {
                    id: label.to_string(),
                    hand: 0,
                    finger: 1,
                    row: current_y as i8,
                    col: current_x as i8,
                    x: current_x,
                    y: current_y,
                    is_stretch: false,
                };

                keys.push(key);

                // Advance cursor
                current_x += current_w;
                // Reset width for next key (KLE standard behavior)
                current_w = 1.0;
            }
        }
        current_y += 1.0;
    }

    // Post-Processing: Simple Hand Detection Split
    // If max_x > 10, assume split layout and assign right hand > 50% width
    if !keys.is_empty() {
        let max_x = keys.iter().fold(0.0f32, |max, k| max.max(k.x));
        let mid_point = max_x / 2.0;

        for k in &mut keys {
            if k.x > mid_point {
                k.hand = 1;
            }
        }
    }

    // Auto-detect slots (Naive)
    let total = keys.len();
    let prime_slots = (0..std::cmp::min(8, total)).collect();
    let med_slots = (8..std::cmp::min(20, total)).collect();
    let low_slots = (20..total).collect();

    let mut geom = KeyboardGeometry {
        keys,
        prime_slots,
        med_slots,
        low_slots,
        home_row: 1, // Assumption
        finger_origins: [[(0.0, 0.0); 5]; 2],
    };

    geom.calculate_origins();
    Ok(geom)
}
