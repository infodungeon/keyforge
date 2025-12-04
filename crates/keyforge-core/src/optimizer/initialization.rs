use crate::core_types::Layout;
use crate::scorer::Scorer;
use fastrand::Rng;
use std::collections::HashSet;

/// Generates a layout using a greedy heuristic:
/// 1. Sort slots by physical quality (reach + finger penalty).
/// 2. Sort characters by frequency.
/// 3. Assign Top Chars -> Best Slots.
/// 4. Respects pins.
pub fn generate_greedy_layout(scorer: &Scorer, rng: &mut Rng, pinned: &[Option<u16>]) -> Layout {
    let key_count = scorer.key_count;
    let mut layout = vec![0u16; key_count];
    let mut filled = vec![false; key_count];
    let mut used_chars = HashSet::new();

    // 1. Apply Pins
    for (i, &p) in pinned.iter().enumerate() {
        if i < key_count {
            if let Some(c) = p {
                layout[i] = c;
                filled[i] = true;
                used_chars.insert(c);
            }
        }
    }

    // 2. Rank Slots (Lower cost is better)
    let mut ranked_slots: Vec<(usize, f32)> = (0..key_count)
        .filter(|&i| !filled[i])
        .map(|i| {
            let cost = scorer.slot_monogram_costs[i];
            (i, cost + rng.f32() * 0.1)
        })
        .collect();

    ranked_slots.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    // 3. Rank Characters (Higher freq is better)
    let mut ranked_chars: Vec<(u16, f32)> = scorer
        .active_chars
        .iter()
        .map(|&idx| {
            let c = idx as u16;
            (c, scorer.char_freqs[idx])
        })
        .filter(|(c, _)| !used_chars.contains(c))
        .collect();

    ranked_chars.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // 4. Assign
    let mut char_iter = ranked_chars.into_iter();

    for (slot_idx, _) in ranked_slots {
        if let Some((char_code, _)) = char_iter.next() {
            layout[slot_idx] = char_code;
        } else {
            layout[slot_idx] = 0;
        }
    }

    layout
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ScoringWeights;
    use crate::geometry::{KeyNode, KeyboardGeometry};
    use crate::scorer::ScorerBuildParams; // FIXED Import
    use std::io::Cursor;

    fn get_test_scorer() -> Scorer {
        let keys = vec![
            KeyNode {
                id: "best".into(),
                hand: 0,
                finger: 1,
                row: 1,
                col: 0,
                x: 0.0,
                y: 0.0,
                w: 1.0,
                h: 1.0,
                is_stretch: false,
            },
            KeyNode {
                id: "worst".into(),
                hand: 0,
                finger: 4,
                row: 0,
                col: 0,
                x: 5.0,
                y: 5.0,
                w: 1.0,
                h: 1.0,
                is_stretch: true,
            },
        ];
        let mut geom = KeyboardGeometry {
            keys,
            prime_slots: vec![],
            med_slots: vec![],
            low_slots: vec![],
            home_row: 1,
            finger_origins: [[(0.0, 0.0); 5]; 2],
        };
        geom.calculate_origins();

        let ngram_data = "a\t10\ne\t1000";
        let cost_data = "From,To,Cost";

        // FIXED: Use new builder params logic for in-memory readers
        ScorerBuildParams::from_readers(
            Cursor::new(cost_data),
            Cursor::new(ngram_data),
            geom,
            Some(ScoringWeights::default()),
            None,
            false,
        )
        .expect("Failed to build scorer")
    }

    #[test]
    fn test_greedy_placement() {
        let scorer = get_test_scorer();
        let mut rng = fastrand::Rng::with_seed(42);
        let pins = vec![None, None];

        let layout = generate_greedy_layout(&scorer, &mut rng, &pins);

        assert_eq!(
            layout[0], b'e' as u16,
            "Greedy failed: Best slot did not get most frequent char"
        );
        assert_eq!(
            layout[1], b'a' as u16,
            "Greedy failed: Worst slot did not get least frequent char"
        );
    }

    #[test]
    fn test_greedy_respects_pins() {
        let scorer = get_test_scorer();
        let mut rng = fastrand::Rng::with_seed(42);

        let mut pins = vec![None, None];
        pins[0] = Some(b'z' as u16);

        let layout = generate_greedy_layout(&scorer, &mut rng, &pins);

        assert_eq!(layout[0], b'z' as u16, "Pin violation");
        assert_eq!(layout[1], b'e' as u16, "Fallback logic failed");
    }
}
