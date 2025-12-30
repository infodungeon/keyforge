// Copyright (c) 2025 KeyForge Contributors
//
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
use keyforge_model::{Corpus, Keyboard, Layout, Rubric};
use keyforge_model::types::{HandIndex, FingerIndex, RowIndex, ColIndex};
use keyforge_model::constants::SCORE_SCALE;

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
        let mut pos_map = vec![65535u16; 65536];
        let limit = layout.keys.len().min(keyboard.keys.len());
        for (i, &code) in layout.keys.iter().enumerate().take(limit) {
            if (code.0 as usize) < pos_map.len() { pos_map[code.0 as usize] = i as u16; }
        }

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

        let fp_keys: Vec<FixedPointKey> = keyboard.keys.iter().map(|k| FixedPointKey {
            x: to_fixed(k.x),
            y: to_fixed(k.y),
            hand: k.hand,
            finger: k.finger,
            row: k.row,
            col: k.col,
        }).collect();

        for &(c1, c2, freq) in &corpus.bigrams {
            let p1 = pos_map[c1 as usize];
            let p2 = pos_map[c2 as usize];
            if p1 != 65535 && p2 != 65535 {
                let k1 = &fp_keys[p1 as usize];
                let k2 = &fp_keys[p2 as usize];
                let mut cost = calculate_pair_cost_int(k1, k2, &fp_rubric);
                for &(o_i, o_j, o_cost) in overrides {
                    if o_i == p1 as usize && o_j == p2 as usize { cost = to_fixed(o_cost); break; }
                }
                total_score = total_score.saturating_add(cost.saturating_mul(freq as i64));
            }
        }

        for &(c1, c2, c3, freq) in &corpus.trigrams {
            let p1 = pos_map[c1 as usize];
            let p2 = pos_map[c2 as usize];
            let p3 = pos_map[c3 as usize];
            if p1 != 65535 && p2 != 65535 && p3 != 65535 {
                let k1 = &fp_keys[p1 as usize];
                let k2 = &fp_keys[p2 as usize];
                let k3 = &fp_keys[p3 as usize];
                let cost = calculate_flow_cost_int(k1, k2, k3, &fp_rubric);
                if cost != 0 { total_score = total_score.saturating_add(cost.saturating_mul(freq as i64)); }
            }
        }
        total_score as f32 / SCORE_SCALE
    }
}

fn to_fixed(val: f32) -> i64 {
    if val.is_nan() { return 0; }
    if val.is_infinite() { return if val.is_sign_positive() { i64::MAX } else { i64::MIN }; }
    let scaled = val * SCORE_SCALE;
    if scaled >= i64::MAX as f32 { i64::MAX } else if scaled <= i64::MIN as f32 { i64::MIN } else { scaled as i64 }
}

struct FixedPointKey {
    x: i64,
    y: i64,
    hand: HandIndex,
    finger: FingerIndex,
    row: RowIndex,
    col: ColIndex,
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

fn calculate_pair_cost_int(k1: &FixedPointKey, k2: &FixedPointKey, rubric: &FixedPointRubric) -> i64 {
    if k1.x == k2.x && k1.y == k2.y { return 0; }
    let dx = (k1.x - k2.x).abs();
    let dy = (k1.y - k2.y).abs();
    let scale_val = SCORE_SCALE as i128;
    let scale_sq = scale_val * scale_val;
    let term_x = (dx as i128).saturating_mul(dx as i128).saturating_mul(rubric.travel_lat as i128) / scale_sq;
    let term_y = (dy as i128).saturating_mul(dy as i128).saturating_mul(rubric.travel_vert as i128) / scale_sq;
    let dist_cost = term_x.saturating_add(term_y);
    let mut cost = if dist_cost > i64::MAX as i128 { i64::MAX } else if dist_cost < i64::MIN as i128 { i64::MIN } else { dist_cost as i64 };

    if k1.hand != k2.hand { return cost; }
    if k1.finger == k2.finger {
        let col_diff = (k1.col - k2.col).abs();
        if col_diff == 1 { cost = cost.saturating_add(rubric.sfb_lateral); } else { cost = cost.saturating_add(rubric.sfb_base); }
        return cost;
    }

    let finger_diff = k1.finger.distance(k2.finger);
    let row_diff = (k1.row - k2.row).abs();
    if finger_diff == 1 && row_diff >= 2 { cost = cost.saturating_add(rubric.finger_effort[k1.finger.as_usize()]); }
    if row_diff == 0 && finger_diff == 1 {
        let col_dist = (k1.col - k2.col).abs();
        if col_dist > 1 { cost = cost.saturating_add(rubric.sfb_lateral); }
    }
    cost
}

fn calculate_flow_cost_int(k1: &FixedPointKey, k2: &FixedPointKey, k3: &FixedPointKey, rubric: &FixedPointRubric) -> i64 {
    if k1.hand != k2.hand || k2.hand != k3.hand { return 0; }
    
    // Redirect check
    if k1.finger == k3.finger && k1.finger != k2.finger { return rubric.redirect; }

    let dir1 = k2.finger.diff(k1.finger);
    let dir2 = k3.finger.diff(k2.finger);
    
    if dir1 == 0 || dir2 == 0 { return 0; }
    if dir1.signum() != dir2.signum() { return rubric.redirect; }
    if dir1 < 0 { return rubric.roll_bonus.saturating_neg(); }
    0
}
