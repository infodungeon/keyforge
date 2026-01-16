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

use crate::kernel::compute::{calculate_swap_delta, score_layout, PhysicsScratch, PosMap};
use crate::kernel::types::ValidatedLayout;
use crate::kernel::EngineContext;
use keyforge_model::{Layout, SwapSuggestion};
use keyforge_model::types::FingerIndex;
use keyforge_model::constants::SCORE_SCALE;

pub fn suggest_swaps(ctx: &EngineContext, layout: &Layout, include_thumbs: bool) -> Vec<SwapSuggestion> {
    // Guardrail: Validate layout before processing
    let validated = match ValidatedLayout::new(&layout.keys, ctx.key_count) {
        Ok(v) => v,
        Err(_) => return vec![], // Invalid layout yields no suggestions
    };

    let mut scratch = PhysicsScratch::new();
    let current_score = score_layout(ctx, &validated, &mut scratch);

    if current_score <= 0 {
        return vec![];
    }

    let mut suggestions = Vec::new();
    let len = layout.keys.len();

    // Create a robust PosMap using scratch buffers
    let pos_map = PosMap::from_scratch(
        &layout.keys,
        ctx.key_count,
        &mut scratch.starts,
        &mut scratch.counts,
        &mut scratch.indices,
        &mut scratch.used_keys,
    );

    for i in 0..len {
        for j in (i + 1)..len {
            if layout.keys[i] == layout.keys[j] {
                continue;
            }

            // Exclude THUMB keys from swap suggestions if requested
            if !include_thumbs && (ctx.fingers[i] == FingerIndex::THUMB || ctx.fingers[j] == FingerIndex::THUMB) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::compiler::Compiler;
    use keyforge_model::{Keyboard, KeyNode, Corpus, Rubric, KeyCode};
    use keyforge_model::types::{HandIndex, FingerIndex, RowIndex, ColIndex};

    fn setup_mock_ctx(size: usize) -> crate::kernel::EngineContext {
        let keys: Vec<_> = (0..size)
            .map(|i| KeyNode {
                index: i,
                label: format!("k{}", i),
                hand: HandIndex((i % 2) as u8),
                finger: FingerIndex((i % 5) as u8),
                row: RowIndex((i / 10) as i8),
                col: ColIndex((i % 10) as i8),
                x: (i % 10) as f32,
                y: (i / 10) as f32,
                ..Default::default()
            })
            .collect();
        let kb = Keyboard::new(keys, 1).unwrap();
        let mut corpus = Corpus::default();
        // Character 10 ('e') is very frequent
        corpus.char_freqs[10] = 1000;
        // Character 32 (' ') is also frequent
        corpus.char_freqs[32] = 2000;
        
        Compiler::compile(&kb, &corpus, &Rubric::default(), &[]).unwrap()
    }

    #[test]
    fn test_suggest_swaps_multi_mapped() {
        let ctx = setup_mock_ctx(30);
        
        // Layout where 'e' (10) is on a very expensive key (index 0)
        // and 'Space' (32) is on two keys: index 28 and 29.
        let mut keys = vec![KeyCode(0); 30];
        for i in 0..30 { keys[i] = KeyCode(i as u16); }
        
        keys[0] = KeyCode(10); // 'e' at worst position
        keys[28] = KeyCode(32); // Space at position 28
        keys[29] = KeyCode(32); // Space at position 29
        
        let layout = Layout::new_unchecked(keys);
        let suggestions = suggest_swaps(&ctx, &layout, true);
        
        // We expect at least one suggestion involving 'e' (index 0) 
        // to be swapped with one of the 'Space' positions or other good positions.
        assert!(!suggestions.is_empty(), "Should suggest improvements");
        
        // Verify that suggestions include swaps with Space (32)
        let has_space_swap = suggestions.iter().any(|s| s.key_a == "32" || s.key_b == "32");
        assert!(has_space_swap, "Should suggest swapping with Space even if it is multi-mapped");
    }
}
