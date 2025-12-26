use keyforge_model::{Corpus, Keyboard, Layout, Rubric};
use keyforge_protocol::constants::SCORE_SCALE;

/// A deterministic scorer that uses pure integer arithmetic.
/// This is slower than the main engine but guarantees identical results across platforms.
/// It should ONLY be used for final verification before submission or storage.
pub struct DeterministicScorer;

impl DeterministicScorer {
    pub fn score(keyboard: &Keyboard, corpus: &Corpus, rubric: &Rubric, layout: &Layout) -> f32 {
        let mut total_score: i64 = 0;

        // 1. Map Layout to Positions
        // pos_map[keycode] = physical_index
        let mut pos_map = vec![255u8; 65536];
        for (i, &code) in layout.keys.iter().enumerate() {
            if (code as usize) < pos_map.len() {
                pos_map[code as usize] = i as u8;
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

            if p1 != 255 && p2 != 255 {
                let k1 = &fp_keys[p1 as usize];
                let k2 = &fp_keys[p2 as usize];

                let cost = calculate_pair_cost_int(k1, k2, &fp_rubric);
                total_score = total_score.saturating_add(cost.saturating_mul(freq as i64));
            }
        }

        // 5. Score Trigrams
        // Note: We apply the same pruning logic as the main engine implicitly
        // by iterating the corpus provided. The corpus itself should be the pruned version if needed.
        for &(c1, c2, c3, freq) in &corpus.trigrams {
            let p1 = pos_map[c1 as usize];
            let p2 = pos_map[c2 as usize];
            let p3 = pos_map[c3 as usize];

            if p1 != 255 && p2 != 255 && p3 != 255 {
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
    (val * SCORE_SCALE) as i64
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

    // Scale down intermediate multiplication to prevent overflow before squaring
    // (dx * weight) / SCALE
    let weighted_dx = (dx * rubric.travel_lat) / (SCORE_SCALE as i64);
    let weighted_dy = (dy * rubric.travel_vert) / (SCORE_SCALE as i64);

    // Result is scaled by 1e6
    let dist_sq = (weighted_dx * weighted_dx + weighted_dy * weighted_dy) / (SCORE_SCALE as i64);

    let mut cost = dist_sq;

    if k1.hand != k2.hand {
        return cost;
    }

    if k1.finger == k2.finger {
        let col_diff = (k1.col - k2.col).abs();
        if col_diff == 1 {
            cost += rubric.sfb_lateral;
        } else {
            cost += rubric.sfb_base;
        }
        return cost;
    }

    let finger_diff = (k1.finger as i8 - k2.finger as i8).abs();
    let row_diff = (k1.row - k2.row).abs();

    if finger_diff == 1 && row_diff >= 2 {
        cost += rubric.finger_effort[k1.finger as usize];
    }

    if row_diff == 0 && finger_diff == 1 {
        let col_dist = (k1.col - k2.col).abs();
        if col_dist > 1 {
            cost += rubric.sfb_lateral;
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
        return -rubric.roll_bonus;
    }

    0
}
