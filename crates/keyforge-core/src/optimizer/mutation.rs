use crate::consts::{KEY_CODE_RANGE, KEY_NOT_FOUND_U8};
use crate::core_types::PosMap;
use fastrand::Rng;
use keyforge_protocol::config::LayoutDefinitions;
use keyforge_protocol::geometry::KeyboardGeometry;

pub fn generate_tiered_layout(
    rng: &mut Rng,
    defs: &LayoutDefinitions,
    geom: &KeyboardGeometry,
    size: usize,
    pinned: &[Option<u16>],
) -> Vec<u16> {
    let mut layout = vec![0u16; size];

    let mut pinned_chars = [false; 256];

    for (i, &p) in pinned.iter().enumerate() {
        if i < size {
            if let Some(c) = p {
                layout[i] = c;
                if c < 256 {
                    pinned_chars[c as usize] = true;
                }
            }
        }
    }

    let filter_pool = |src: &str| -> Vec<u16> {
        src.as_bytes()
            .iter()
            .map(|&b| b as u16)
            .filter(|&c| c >= 256 || !pinned_chars[c as usize])
            .collect()
    };

    let mut high = filter_pool(&defs.tier_high_chars);
    let mut med = filter_pool(&defs.tier_med_chars);
    let mut low = filter_pool(&defs.tier_low_chars);

    rng.shuffle(&mut high);
    rng.shuffle(&mut med);
    rng.shuffle(&mut low);

    let mut fill_slot = |slot: usize, pools: &mut [&mut Vec<u16>]| {
        if slot < size && layout[slot] == 0 {
            for pool in pools {
                if let Some(c) = pool.pop() {
                    layout[slot] = c;
                    return;
                }
            }
            layout[slot] = 0;
        }
    };

    for &slot in &geom.prime_slots {
        fill_slot(slot, &mut [&mut high, &mut med]);
    }
    for &slot in &geom.med_slots {
        fill_slot(slot, &mut [&mut med, &mut low]);
    }
    for &slot in &geom.low_slots {
        fill_slot(slot, &mut [&mut low, &mut med, &mut high]);
    }

    layout
}

pub fn build_pos_map(layout: &[u16]) -> PosMap {
    let mut map = Box::new([KEY_NOT_FOUND_U8; KEY_CODE_RANGE]);
    for (i, &code) in layout.iter().enumerate() {
        if code != 0 {
            map[code as usize] = i as u8;
            if code < 128 {
                let b = code as u8;
                if b.is_ascii_uppercase() {
                    map[b.to_ascii_lowercase() as usize] = i as u8;
                } else if b.is_ascii_lowercase() {
                    map[b.to_ascii_uppercase() as usize] = i as u8;
                }
            }
        }
    }
    map
}

pub fn fails_sanity(
    pos_map: &[u8; KEY_CODE_RANGE],
    critical_bigrams: &[[u8; 2]],
    geom: &KeyboardGeometry,
) -> bool {
    for pair in critical_bigrams {
        let p1 = pos_map[pair[0] as usize];
        let p2 = pos_map[pair[1] as usize];

        if p1 == KEY_NOT_FOUND_U8 || p2 == KEY_NOT_FOUND_U8 {
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

#[cfg(test)]
mod tests {
    use super::*;
    use keyforge_protocol::geometry::KeyNode;

    #[test]
    fn test_pos_map_basic() {
        let layout = vec![65, 66, 0]; // A, B, Empty
        let map = build_pos_map(&layout);

        assert_eq!(map[65], 0);
        assert_eq!(map[97], 0); // Case folding (a -> A)
        assert_eq!(map[66], 1);
        assert_eq!(map[67], KEY_NOT_FOUND_U8);
    }

    #[test]
    fn test_fails_sanity_on_sfb() {
        // Setup: Q (index 0) and A (index 1) are on same finger
        let k1 = KeyNode {
            hand: 0,
            finger: 1,
            ..Default::default()
        };
        let k2 = KeyNode {
            hand: 0,
            finger: 1,
            ..Default::default()
        };
        let geom = KeyboardGeometry {
            keys: vec![k1, k2],
            ..Default::default()
        };

        // Map: 't' at 0, 'h' at 1
        let mut map = Box::new([KEY_NOT_FOUND_U8; KEY_CODE_RANGE]);
        map[b't' as usize] = 0;
        map[b'h' as usize] = 1;

        // Critical: "th"
        let crit = vec![[b't', b'h']];

        assert!(fails_sanity(&map, &crit, &geom));
    }
}
