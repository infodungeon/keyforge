// libs/keyforge-physics/src/analysis/heuristics.rs

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

use crate::kernel::compute::{calculate_swap_delta, score_layout};
use crate::kernel::types::ValidatedLayout;
use crate::kernel::EngineContext;
use keyforge_model::{Layout, SwapSuggestion};
use keyforge_model::constants::SCORE_SCALE;

pub fn suggest_swaps(ctx: &EngineContext, layout: &Layout) -> Vec<SwapSuggestion> {
    // Guardrail: Validate layout before processing
    let validated = match ValidatedLayout::new(&layout.keys, ctx.key_count) {
        Ok(v) => v,
        Err(_) => return vec![], // Invalid layout yields no suggestions
    };

    let mut scratch = crate::kernel::compute::PhysicsScratch::new();
    let current_score = score_layout(ctx, &validated, &mut scratch);

    if current_score <= 0 {
        return vec![];
    }

    let mut suggestions = Vec::new();
    let len = layout.keys.len();

    // Use a temporary pos_map for delta calculations
    let mut pos_map = vec![65535u16; 65536];
    for (i, &code) in layout.keys.iter().enumerate() {
        pos_map[code.0 as usize] = i as u16;
    }

    for i in 0..len {
        for j in (i + 1)..len {
            if layout.keys[i] == layout.keys[j] {
                continue;
            }

            let delta = calculate_swap_delta(ctx, &validated, &pos_map, i, j);

            if delta < 0 {
                let improvement = delta.abs() as f32 / SCORE_SCALE;
                let current_f32 = current_score as f32 / SCORE_SCALE;

                let pct = if current_f32 > f32::EPSILON {
                    (improvement / current_f32) * 100.0
                } else {
                    0.0
                };

                if pct > 0.1 && pct.is_finite() {
                    suggestions.push(SwapSuggestion {
                        index_a: i,
                        index_b: j,
                        key_a: format!("{}", layout.keys[i]),
                        key_b: format!("{}", layout.keys[j]),
                        score_delta: improvement,
                        improvement_pct: pct,
                    });
                }
            }
        }
    }

    suggestions.sort_by(|a, b| b.improvement_pct.total_cmp(&a.improvement_pct));
    suggestions.truncate(5);
    suggestions
}
