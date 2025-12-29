use keyforge_model::{Corpus, Keyboard, Layout, Rubric};
use keyforge_protocol::constants::SCORE_SCALE;

/// A deterministic scorer that uses pure integer arithmetic.
/// This is slower than the main engine but guarantees identical results across platforms.
/// It should ONLY be used for final verification before submission or storage.
pub struct DeterministicScorer;

impl DeterministicScorer {
    pub fn score(
        keyboard: &Keyboard,
        corpus: &Corpus,
        rubric: &Rubric,
        layout: &Layout,
        overrides: &[(usize, usize, f32)],
    ) -> f32 {
        let mut total_score: i64 = 0;

        // 1. Map Layout to Positions
        // pos_map[keycode] = physical_index
        let mut pos_map = vec![65535u16; 65536];
        let limit = layout.keys.len().min(keyboard.keys.len());
        for (i, &code) in layout.keys.iter().enumerate().take(limit) {
            if (code as usize) < pos_map.len() {
                pos_map[code as usize] = i as u16;
            }
        }

        // 2. Convert Rubric to Fixed Point
        let fp_rubric = FixedPointRubric {
            travel_lat: to_fixed(rubric.travel_lat),
            travel_vert: to_fixed(rubric.travel_vert),
            sfb_base: to_fixed(rubric.sfb_base),
            sfb_lateral: to_fixed(rubric.sfb_lateral),
            finger_effort: [
                to_fixed(rubric.finger_effort[0]),
                to_fixed(rubric.finger_effort[1]),
                to_fixed(rubric.finger_effort[2]),
                to_fixed(rubric.finger_effort[3]),
                to_fixed(rubric.finger_effort[4]),
            ],
            redirect: to_fixed(rubric.redirect),
            roll_bonus: to_fixed(rubric.roll_bonus),
        };

        // 3. Convert Keyboard to Fixed Point
        let fp_keys: Vec<FixedPointKey> = keyboard
            .keys
            .iter()
            .map(|k| FixedPointKey {
                x: to_fixed(k.x),
                y: to_fixed(k.y),
                hand: k.hand,
                finger: k.finger,
                row: k.row,
                col: k.col,
            })
            .collect();

        // 4. Score Bigrams
        for &(c1, c2, freq) in &corpus.bigrams {
            let p1 = pos_map[c1 as usize];
            let p2 = pos_map[c2 as usize];

            if p1 != 65535 && p2 != 65535 {
                let k1 = &fp_keys[p1 as usize];
                let k2 = &fp_keys[p2 as usize];

                let mut cost = calculate_pair_cost_int(k1, k2, &fp_rubric);
                
                // Apply overrides
                for &(o_i, o_j, o_cost) in overrides {
                    if o_i == p1 as usize && o_j == p2 as usize {
                        cost = to_fixed(o_cost);
                        break;
                    }
                }

                total_score = total_score.saturating_add(cost.saturating_mul(freq as i64));
            }
        }

        // 5. Score Trigrams
        for &(c1, c2, c3, freq) in &corpus.trigrams {
            let p1 = pos_map[c1 as usize];
            let p2 = pos_map[c2 as usize];
            let p3 = pos_map[c3 as usize];

            if p1 != 65535 && p2 != 65535 && p3 != 65535 {
                let k1 = &fp_keys[p1 as usize];
                let k2 = &fp_keys[p2 as usize];
                let k3 = &fp_keys[p3 as usize];

                let cost = calculate_flow_cost_int(k1, k2, k3, &fp_rubric);
                if cost != 0 {
                    total_score = total_score.saturating_add(cost.saturating_mul(freq as i64));
                }
            }
        }

        total_score as f32 / SCORE_SCALE
    }
}

// --- Internal Fixed Point Logic ---

fn to_fixed(val: f32) -> i64 {
    if val.is_nan() {
        return 0;
    }
    if val.is_infinite() {
        return if val.is_sign_positive() {
            i64::MAX
        } else {
            i64::MIN
        };
    }
    let scaled = val * SCORE_SCALE;
    if scaled >= i64::MAX as f32 {
        i64::MAX
    } else if scaled <= i64::MIN as f32 {
        i64::MIN
    } else {
        scaled as i64
    }
}

struct FixedPointKey {
    x: i64,
    y: i64,
    hand: u8,
    finger: u8,
    row: i8,
    col: i8,
}

struct FixedPointRubric {
    travel_lat: i64,
    travel_vert: i64,
    sfb_base: i64,
    sfb_lateral: i64,
    finger_effort: [i64; 5],
    redirect: i64,
    roll_bonus: i64,
}

fn calculate_pair_cost_int(
    k1: &FixedPointKey,
    k2: &FixedPointKey,
    rubric: &FixedPointRubric,
) -> i64 {
    if k1.x == k2.x && k1.y == k2.y {
        return 0;
    }

    let dx = (k1.x - k2.x).abs();
    let dy = (k1.y - k2.y).abs();

    let scale_val = SCORE_SCALE as i128;
    let scale_sq = scale_val * scale_val;

    let term_x = (dx as i128)
        .saturating_mul(dx as i128)
        .saturating_mul(rubric.travel_lat as i128)
        / scale_sq;
    let term_y = (dy as i128)
        .saturating_mul(dy as i128)
        .saturating_mul(rubric.travel_vert as i128)
        / scale_sq;

    let dist_cost = term_x.saturating_add(term_y);

    let mut cost = if dist_cost > i64::MAX as i128 {
        i64::MAX
    } else if dist_cost < i64::MIN as i128 {
        i64::MIN
    } else {
        dist_cost as i64
    };

    if k1.hand != k2.hand {
        return cost;
    }

    if k1.finger == k2.finger {
        let col_diff = (k1.col - k2.col).abs();
        if col_diff == 1 {
            cost = cost.saturating_add(rubric.sfb_lateral);
        } else {
            cost = cost.saturating_add(rubric.sfb_base);
        }
        return cost;
    }

    let finger_diff = (k1.finger as i8 - k2.finger as i8).abs();
    let row_diff = (k1.row - k2.row).abs();

    if finger_diff == 1 && row_diff >= 2 {
        cost = cost.saturating_add(rubric.finger_effort[k1.finger as usize]);
    }

    if row_diff == 0 && finger_diff == 1 {
        let col_dist = (k1.col - k2.col).abs();
        if col_dist > 1 {
            cost = cost.saturating_add(rubric.sfb_lateral);
        }
    }

    cost
}

fn calculate_flow_cost_int(
    k1: &FixedPointKey,
    k2: &FixedPointKey,
    k3: &FixedPointKey,
    rubric: &FixedPointRubric,
) -> i64 {
    if k1.hand != k2.hand || k2.hand != k3.hand {
        return 0;
    }

    let f1 = k1.finger as i8;
    let f2 = k2.finger as i8;
    let f3 = k3.finger as i8;

    if f1 == f3 && f1 != f2 {
        return rubric.redirect;
    }

    let dir1 = f2 - f1;
    let dir2 = f3 - f2;

    if dir1 == 0 || dir2 == 0 {
        return 0;
    }

    if dir1.signum() != dir2.signum() {
        return rubric.redirect;
    }

    if dir1 < 0 {
        return rubric.roll_bonus.saturating_neg();
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyforge_model::{KeyNode, Keyboard, Layout, Rubric, Corpus};

    fn setup_kb() -> Keyboard {
        let keys: Vec<KeyNode> = (0..5)
            .map(|i| KeyNode {
                id: i,
                label: format!("k{}", i),
                hand: 0,
                finger: i as u8,
                row: 0,
                col: i as i8,
                x: i as f32,
                y: 0.0,
                is_home: true,
            })
            .collect();
        Keyboard::new(keys, 0)
    }

    #[test]
    fn test_verify_nan_handling() {
        let kb = setup_kb();
        let layout = Layout::new_unchecked(vec![0, 1, 2, 3, 4]);
        let mut corpus = Corpus::default();
        corpus.bigrams.push((0, 1, 100));

        let rubric = Rubric {
            travel_lat: f32::NAN,
            ..Rubric::default()
        };

        let score = DeterministicScorer::score(&kb, &corpus, &rubric, &layout, &[]);
        assert!(score >= 0.0);
        assert!(!score.is_nan());
    }

    #[test]
    fn test_verify_saturation_max() {
        let kb = setup_kb();
        let layout = Layout::new_unchecked(vec![0, 1, 2, 3, 4]);
        let mut corpus = Corpus::default();
        corpus.bigrams.push((0, 1, 1));

        let rubric = Rubric {
            travel_lat: 1e30,
            ..Rubric::default()
        };

        let score = DeterministicScorer::score(&kb, &corpus, &rubric, &layout, &[]);
        assert!(score > 1_000_000.0);
        assert!(score.is_finite());
    }

    #[test]
    fn test_verify_saturation_min() {
        let kb = setup_kb();
        let layout = Layout::new_unchecked(vec![0, 1, 2, 3, 4]);
        let mut corpus = Corpus::default();
        corpus.bigrams.push((0, 1, 1));

        let rubric = Rubric {
            travel_lat: -1e30,
            ..Rubric::default()
        };

        let score = DeterministicScorer::score(&kb, &corpus, &rubric, &layout, &[]);
        assert!(score < -1_000_000.0);
        assert!(score.is_finite());
    }

    #[test]
    fn test_verify_trigram_redirects() {
        let kb = setup_kb();
        let layout = Layout::new_unchecked(vec![0, 1, 2, 3, 4]);
        let mut corpus = Corpus::default();
        corpus.trigrams.push((0, 1, 0, 100));

        let rubric = Rubric {
            redirect: 10.0,
            ..Rubric::default()
        };

        let score = DeterministicScorer::score(&kb, &corpus, &rubric, &layout, &[]);
        assert!(score > 0.0);
    }

    #[test]
    fn test_verify_trigram_rolls_negative_dir() {
        let kb = setup_kb();
        let layout = Layout::new_unchecked(vec![0, 1, 2, 3, 4]);
        let mut corpus = Corpus::default();
        corpus.trigrams.push((2, 1, 0, 100));

        let rubric = Rubric {
            roll_bonus: 5.0,
            ..Rubric::default()
        };

        let score = DeterministicScorer::score(&kb, &corpus, &rubric, &layout, &[]);
        assert!(score < 0.0);
    }

    #[test]
    fn test_verify_trigram_zero_dir() {
        let kb = setup_kb();
        let layout = Layout::new_unchecked(vec![0, 1, 2, 3, 4]);
        let mut corpus = Corpus::default();
        corpus.trigrams.push((0, 0, 1, 100));

        let rubric = Rubric {
            travel_lat: 0.0,
            travel_vert: 0.0,
            sfb_base: 0.0,
            sfb_lateral: 0.0,
            redirect: 10.0,
            roll_bonus: 10.0,
            ..Rubric::default()
        };
        let score = DeterministicScorer::score(&kb, &corpus, &rubric, &layout, &[]);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_verify_trigram_redirect_signs() {
        let kb = setup_kb();
        let layout = Layout::new_unchecked(vec![0, 1, 2, 3, 4]);
        let mut corpus = Corpus::default();
        corpus.trigrams.push((0, 2, 1, 100));

        let rubric = Rubric {
            redirect: 10.0,
            travel_lat: 0.0,
            travel_vert: 0.0,
            sfb_base: 0.0,
            sfb_lateral: 0.0,
            ..Rubric::default()
        };

        let score = DeterministicScorer::score(&kb, &corpus, &rubric, &layout, &[]);
        assert!(score > 0.0);
    }

    #[test]
    fn test_verify_extreme_coordinates() {
        let mut keys = Vec::new();
        keys.push(KeyNode {
            id: 0,
            label: "k1".to_string(),
            hand: 0,
            finger: 1,
            row: 0,
            col: 0,
            x: 0.0,
            y: 0.0,
            is_home: false,
        });
        keys.push(KeyNode {
            id: 1,
            label: "k2".to_string(),
            hand: 0,
            finger: 2,
            row: 0,
            col: 1,
            x: 30000.0,
            y: 0.0,
            is_home: false,
        });
        let kb = Keyboard::new(keys, 0);

        let mut corpus = Corpus::default();
        corpus.bigrams.push((0, 1, 100));

        let mut rubric = Rubric::default();
        rubric.travel_lat = 1_000_000_000_000.0;

        let layout = Layout::new_unchecked(vec![0, 1]);

        let score = DeterministicScorer::score(&kb, &corpus, &rubric, &layout, &[]);
        assert!(score > 0.0);
    }

    #[test]
    fn test_math_boundaries_neg_infinity() {
        let kb = setup_kb();
        let layout = Layout::new_unchecked(vec![97, 98, 99, 100, 101]);

        let mut corpus = Corpus::default();
        corpus.bigrams.push((97, 98, 1000));

        let rubric = Rubric {
            travel_lat: f32::NEG_INFINITY,
            ..Rubric::default()
        };

        let det_score = DeterministicScorer::score(&kb, &corpus, &rubric, &layout, &[]);
        assert!(det_score < -1_000_000.0);
        assert!(det_score.is_finite());
    }
}
