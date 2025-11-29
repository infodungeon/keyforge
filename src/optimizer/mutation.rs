// ===== keyforge/src/optimizer/mutation.rs =====
use crate::config::LayoutDefinitions;
use crate::geometry::KeyboardGeometry;
use fastrand::Rng;

/// Generates a random layout respecting the tier definition in Config
pub fn generate_tiered_layout(
    rng: &mut Rng,
    defs: &LayoutDefinitions,
    geom: &KeyboardGeometry,
    size: usize, // NEW: Explicit Size
) -> Vec<u8> {
    let mut layout = vec![0u8; size];

    // Convert configuration strings to byte vectors for shuffling
    let mut high = defs.tier_high_chars.as_bytes().to_vec();
    let mut med = defs.tier_med_chars.as_bytes().to_vec();
    let mut low = defs.tier_low_chars.as_bytes().to_vec();

    rng.shuffle(&mut high);
    rng.shuffle(&mut med);
    rng.shuffle(&mut low);

    // 1. Fill Prime Slots (Top Priority)
    for &slot in &geom.prime_slots {
        if slot < size {
            if !high.is_empty() {
                layout[slot] = high.pop().unwrap();
            } else if !med.is_empty() {
                layout[slot] = med.pop().unwrap();
            }
        }
    }

    // 2. Fill Medium Slots
    for &slot in &geom.med_slots {
        if slot < size && layout[slot] == 0 {
            if !med.is_empty() {
                layout[slot] = med.pop().unwrap();
            } else if !low.is_empty() {
                layout[slot] = low.pop().unwrap();
            }
        }
    }

    // 3. Fill Low Slots
    for &slot in &geom.low_slots {
        if slot < size && layout[slot] == 0 {
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

pub fn build_pos_map(layout: &[u8]) -> [u8; 256] {
    let mut map = [255u8; 256];
    for (i, &byte) in layout.iter().enumerate() {
        if byte != 0 {
            map[byte as usize] = i as u8;
            if byte.is_ascii_uppercase() {
                map[byte.to_ascii_lowercase() as usize] = i as u8;
            } else if byte.is_ascii_lowercase() {
                map[byte.to_ascii_uppercase() as usize] = i as u8;
            }
        }
    }
    map
}

pub fn fails_sanity(
    pos_map: &[u8; 256],
    critical_bigrams: &[[u8; 2]],
    geom: &KeyboardGeometry,
) -> bool {
    for pair in critical_bigrams {
        let p1 = pos_map[pair[0] as usize];
        let p2 = pos_map[pair[1] as usize];

        if p1 == 255 || p2 == 255 {
            continue;
        }

        let info1 = &geom.keys[p1 as usize];
        let info2 = &geom.keys[p2 as usize];

        if info1.hand == info2.hand && info1.finger == info2.finger {
            return true;
        }
    }
    false
}
