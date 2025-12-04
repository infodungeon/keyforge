// ===== keyforge/crates/keyforge-core/src/geometry/kle.rs =====
use super::{KeyNode, KeyboardGeometry};
use std::error::Error;

/// Parses raw KLE JSON content into KeyForge Geometry using the kle-serial crate.
pub fn parse_kle_json(content: &str) -> Result<KeyboardGeometry, Box<dyn Error>> {
    // 1. Use the crate to parse the messy KLE format
    let keyboard: kle_serial::Keyboard = serde_json::from_str(content)?;

    let mut keys = Vec::new();

    // 2. Map crate types to KeyForge types
    // FIXED: Use enumerate() instead of manual counter
    for (current_id, key) in keyboard.keys.into_iter().enumerate() {
        // Simple heuristic for hand splitting (left vs right of center)
        let hand = if key.x > 10.0 { 1 } else { 0 };

        let finger = 1;

        // Flatten unwraps Option<Legend>, find looks for non-empty text
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
            is_stretch: false,
        };

        keys.push(node);
    }

    let total = keys.len();
    let prime_slots = (0..std::cmp::min(8, total)).collect();
    let med_slots = (8..std::cmp::min(20, total)).collect();
    let low_slots = (20..total).collect();

    let mut geom = KeyboardGeometry {
        keys,
        prime_slots,
        med_slots,
        low_slots,
        home_row: 1,
        finger_origins: [[(0.0, 0.0); 5]; 2],
    };

    geom.calculate_origins();
    Ok(geom)
}
