use crate::config::LayoutDefinitions;
use crate::geometry::KeyboardGeometry;
use fastrand::Rng;

/// Generates a random layout respecting the tier definition in Config
pub fn generate_tiered_layout(
    rng: &mut Rng,
    defs: &LayoutDefinitions,
    geom: &KeyboardGeometry,
) -> [u8; 30] {
    let mut layout = [0u8; 30];

    // Convert configuration strings to byte vectors for shuffling
    let mut high = defs.tier_high_chars.as_bytes().to_vec();
    let mut med = defs.tier_med_chars.as_bytes().to_vec();
    let mut low = defs.tier_low_chars.as_bytes().to_vec();

    rng.shuffle(&mut high);
    rng.shuffle(&mut med);
    rng.shuffle(&mut low);

    // 1. Fill Prime Slots (Top Priority)
    // DYNAMIC: Use geometry prime slots
    for &slot in &geom.prime_slots {
        if !high.is_empty() {
            layout[slot] = high.pop().unwrap();
        } else if !med.is_empty() {
            layout[slot] = med.pop().unwrap();
        }
    }

    // 2. Fill Medium Slots
    for &slot in &geom.med_slots {
        // Only fill if not already taken by a high char (if high overflowed)
        if layout[slot] == 0 {
            if !med.is_empty() {
                layout[slot] = med.pop().unwrap();
            } else if !low.is_empty() {
                layout[slot] = low.pop().unwrap();
            }
        }
    }

    // 3. Fill Low Slots
    for &slot in &geom.low_slots {
        if layout[slot] == 0 {
            if !low.is_empty() {
                layout[slot] = low.pop().unwrap();
            } else if !med.is_empty() {
                layout[slot] = med.pop().unwrap();
            } else if !high.is_empty() {
                layout[slot] = high.pop().unwrap();
            }
        }
    }
    layout
}

pub fn build_pos_map(layout: &[u8; 30]) -> [u8; 256] {
    let mut map = [255u8; 256];
    for (i, &byte) in layout.iter().enumerate() {
        map[byte as usize] = i as u8;
    }
    map
}

/// Checks if the layout violates critical bigram constraints (SFBs on critical pairs)
pub fn fails_sanity(
    pos_map: &[u8; 256],
    critical_bigrams: &[[u8; 2]],
    geom: &KeyboardGeometry,
) -> bool {
    for pair in critical_bigrams {
        let p1 = pos_map[pair[0] as usize];
        let p2 = pos_map[pair[1] as usize];

        // Skip if one char isn't in the layout (shouldn't happen in search, but safe)
        if p1 == 255 || p2 == 255 {
            continue;
        }

        // DYNAMIC: Access key info via geometry
        let info1 = &geom.keys[p1 as usize];
        let info2 = &geom.keys[p2 as usize];

        // Sanity Check: Same hand, same finger = SFB (Bad for critical pairs)
        if info1.hand == info2.hand && info1.finger == info2.finger {
            return true;
        }
    }
    false
}
