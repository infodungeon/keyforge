use crate::kernel::compute::{calculate_swap_delta, score_layout};
use crate::kernel::types::ValidatedLayout;
use crate::kernel::EngineContext;
use keyforge_model::{Layout, SwapSuggestion};
use keyforge_protocol::constants::SCORE_SCALE;

pub fn suggest_swaps(ctx: &EngineContext, layout: &Layout) -> Vec<SwapSuggestion> {
    // Guardrail: Validate layout before processing
    let validated = match ValidatedLayout::new(&layout.keys, ctx.key_count) {
        Ok(v) => v,
        Err(_) => return vec![], // Invalid layout yields no suggestions
    };

    let mut pos_map = vec![65535u16; 65536];
    let current_score = score_layout(ctx, &validated, &mut pos_map);

    if current_score <= 0 {
        return vec![];
    }

    let mut suggestions = Vec::new();
    let len = layout.keys.len();

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
                        key_a: format!("{}", i),
                        key_b: format!("{}", j),
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
