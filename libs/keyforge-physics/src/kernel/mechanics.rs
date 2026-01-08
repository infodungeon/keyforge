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

use super::types::{DistanceSquared, KeyIndex};
use keyforge_model::{Keyboard, Rubric};

pub fn calculate_pair_cost(kb: &Keyboard, rubric: &Rubric, i: KeyIndex, j: KeyIndex) -> f32 {
    let i_idx = usize::from(i);
    let j_idx = usize::from(j);

    if i_idx == j_idx { return 0.0; }

    let k1 = &kb.keys[i_idx];
    let k2 = &kb.keys[j_idx];

    let h1 = k1.hand;
    let h2 = k2.hand;
    let f1 = k1.finger;
    let f2 = k2.finger;

    let dx = (k1.x - k2.x).abs();
    let dy = (k1.y - k2.y).abs();
    let dist_raw = (dx * dx * rubric.travel_lat) + (dy * dy * rubric.travel_vert);
    let dist_sq = DistanceSquared::new(dist_raw);
    let mut cost = dist_sq.as_f32();

    if h1 != h2 { return cost; }

    if f1 == f2 {
        let col_diff = (k1.col - k2.col).abs();
        if col_diff == 1 { cost += rubric.sfb_lateral; } else { cost += rubric.sfb_base; }
        return cost;
    }

    let finger_diff = f1.distance(f2);
    
    let row_diff = (k1.row - k2.row).abs();

    if finger_diff == 1 && row_diff >= 2 {
        cost += rubric.finger_effort[f1.as_usize()];
    }

    if row_diff == 0 && finger_diff == 1 {
        let col_dist = (k1.col - k2.col).abs();
        if col_dist > 1 { cost += rubric.sfb_lateral; }
    }
    cost
}
