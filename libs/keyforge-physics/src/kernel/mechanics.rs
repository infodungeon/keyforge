// libs/keyforge-physics/src/kernel/mechanics.rs

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

use super::types::{FingerIndex, KeyIndex};
use keyforge_model::{Keyboard, Rubric};

pub fn calculate_pair_cost(kb: &Keyboard, rubric: &Rubric, i: KeyIndex, j: KeyIndex) -> f32 {
    let i_idx = usize::from(i);
    let j_idx = usize::from(j);

    if i_idx == j_idx {
        return 0.0;
    }

    let k1 = &kb.keys[i_idx];
    let k2 = &kb.keys[j_idx];

    let h1 = k1.hand;
    let h2 = k2.hand;
    let f1 = k1.finger;
    let f2 = k2.finger;

    let (dx2, dy2) = kb.spatial_cache[i_idx * kb.keys.len() + j_idx];
    let dist_raw = (dx2 * rubric.travel_lat) + (dy2 * rubric.travel_vert);

    if h1 != h2 {
        return 0.0;
    }

    let mut cost = dist_raw;

    if f1 == f2 {
        // SFB Correction:
        // We counted Reach(K1) + Reach(K2) in Monograms.
        // We want Reach(K1) + Dist(K1, K2).
        // So we subtract Reach(K2).

        let mut reach_k2 = 0.0;
        if let Some(origin) = kb
            .finger_origins
            .get(k2.hand.as_usize())
            .and_then(|h| h.get(k2.finger.as_usize()))
        {
            let odx = (k2.x - origin.0).abs();
            let ody = (k2.y - origin.1).abs();
            reach_k2 = (odx * odx * rubric.travel_lat) + (ody * ody * rubric.travel_vert);
        }

        cost -= reach_k2;

        let row_diff = (k1.row - k2.row).abs();
        let col_diff = (k1.col - k2.col).abs();

        if col_diff == 1 {
            // Lateral SFB
            if f1.is_weak() {
                cost += rubric.sfb_lateral_weak;
            } else {
                cost += rubric.sfb_lateral;
            }
        } else if col_diff > 1 {
            // Diagonal SFB
            cost += rubric.sfb_diagonal;
        } else if row_diff >= rubric.threshold_sfb_long_row_diff {
            // Long SFB (Vertical Jump)
            cost += rubric.sfb_long;
        } else {
            // Standard SFB
            cost += rubric.sfb_base;
        }
        return cost;
    }

    let finger_diff = f1.distance(f2);
    let row_diff = (k1.row - k2.row).abs();

    // Scissor detection (Adjacent fingers, large row difference)
    // Thumbs (0) are excluded from scissor/stretch detection
    if finger_diff == 1 && f1 != FingerIndex::THUMB && f2 != FingerIndex::THUMB {
        if row_diff >= rubric.threshold_scissor_row_diff {
            cost += rubric.penalty_scissor;
        } else if row_diff == 0 {
            // Lateral Stretch (Adjacent fingers, same row, spread out)
            let col_diff = (k1.col - k2.col).abs();
            if col_diff > 1 {
                cost += rubric.sfb_lateral;
            }
        }
    }

    cost
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyforge_model::types::{ColIndex, FingerIndex, HandIndex, RowIndex};
    use keyforge_model::KeyNode;

    fn setup_kb_pair() -> Keyboard {
        let keys = vec![
            KeyNode {
                index: 0,
                hand: HandIndex(0),
                finger: FingerIndex(1),
                row: RowIndex(0),
                col: ColIndex(0),
                x: 0.0,
                y: 0.0,
                is_home: true,
                ..Default::default()
            },
            KeyNode {
                index: 1,
                hand: HandIndex(0),
                finger: FingerIndex(1),
                row: RowIndex(1),
                col: ColIndex(0),
                x: 0.0,
                y: 1.0,
                is_home: false,
                ..Default::default()
            },
            KeyNode {
                index: 2,
                hand: HandIndex(0),
                finger: FingerIndex(2),
                row: RowIndex(1),
                col: ColIndex(1),
                x: 1.0,
                y: 1.0,
                is_home: false,
                ..Default::default()
            },
        ];
        Keyboard::new(keys, 0).unwrap()
    }

    #[test]
    fn test_calculate_pair_cost_sfb() {
        let kb = setup_kb_pair();
        let rubric = Rubric::default();

        let cost = calculate_pair_cost(&kb, &rubric, KeyIndex(0), KeyIndex(1));
        assert!(cost >= rubric.sfb_base, "SFB should be penalized");
    }

    #[test]
    fn test_calculate_pair_cost_different_hands() {
        let keys = vec![
            KeyNode {
                index: 0,
                hand: HandIndex(0),
                finger: FingerIndex(1),
                ..Default::default()
            },
            KeyNode {
                index: 1,
                hand: HandIndex(1),
                finger: FingerIndex(1),
                ..Default::default()
            },
        ];
        let kb = Keyboard::new(keys, 0).unwrap();
        let rubric = Rubric::default();

        let cost = calculate_pair_cost(&kb, &rubric, KeyIndex(0), KeyIndex(1));
        assert_eq!(cost, 0.0, "Different hands should have 0 cost");
    }
}
