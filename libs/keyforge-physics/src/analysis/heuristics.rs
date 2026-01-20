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
use keyforge_model::constants::SCORE_SCALE;
use keyforge_model::types::FingerIndex;
use keyforge_model::{Layout, SwapSuggestion};

pub fn suggest_swaps(
    ctx: &EngineContext,
    layout: &Layout,
    include_thumbs: bool,
) -> Vec<SwapSuggestion> {
    // Guardrail: Validate layout before processing
    let Ok(validated) = ValidatedLayout::new(&layout.keys, ctx.key_count) else {
        return vec![]; // Invalid layout yields no suggestions
    };

    let mut scratch = PhysicsScratch::new();
    let Ok(current_score) = score_layout(ctx, &validated, &mut scratch) else {
        return vec![]; // Scoring failure yields no suggestions
    };

    if current_score <= 0 {
        return vec![];
    }

    let mut suggestions = Vec::new();
    let len = layout.keys.len();

    // Create a robust PosMap using scratch buffers
    let pos_map = PosMap::from_scratch(
        &layout.keys,
        ctx.key_count,
        scratch.starts.as_mut_slice(),
        scratch.counts.as_mut_slice(),
        scratch.indices.as_mut_slice(),
        scratch.current_offsets.as_mut_slice(),
        &mut scratch.used_keys,
    );

    for i in 0..len {
        for j in (i + 1)..len {
            if layout.keys[i] == layout.keys[j] {
                continue;
            }

            // Exclude THUMB keys from swap suggestions if requested
            if !include_thumbs
                && (ctx.fingers[i] == FingerIndex::THUMB || ctx.fingers[j] == FingerIndex::THUMB)
            {
                continue;
            }

            let delta = calculate_swap_delta(ctx, &validated, &pos_map, i, j);

            if delta < 0 {
                #[allow(clippy::cast_precision_loss)]
                let improvement = delta.unsigned_abs() as f32 / SCORE_SCALE;
                #[allow(clippy::cast_precision_loss)]
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
    use crate::{EngineFactory, Compiler};
    use keyforge_model::{
        types::{FingerIndex, HandIndex, KeyCode, RowIndex, ColIndex},
        Corpus, CostModel, KeyNode, Keyboard, Layout, Rubric,
    };

    fn mock_cost_model() -> CostModel {
        // We define distinct costs for rows to create a gradient.
        // r0 (home row) = 1.0 (Best)
        // r1 (top row) = 2.0
        // r2 (bottom row) = 3.0
        let json = r#"{
            "meta": { "version": "2.0", "description": "Test", "unit": "pts" },
            "models": {
                "model_a_row_staggered": {
                    "description": "Test Model",
                    "static_costs": {
                        "universal_hand": {
                            "thumb": { "pos_1": 1.0 },
                            "index": { "base": { "r0": 1.0, "r1": 2.0, "r2": 3.0 } },
                            "middle": { "base": { "r0": 1.0, "r1": 2.0, "r2": 3.0 } },
                            "ring": { "base": { "r0": 1.0, "r1": 2.0, "r2": 3.0 } },
                            "pinky": { "base": { "r0": 1.0, "r1": 2.0, "r2": 3.0 } }
                        }
                    }
                }
            },
            "dynamic_rules": { "sequence_modifiers": {}, "penalties": {}, "constraints": {} }
        }"#;
        serde_json::from_str(json).unwrap()
    }

    fn setup_mock_ctx(size: usize) -> crate::kernel::EngineContext {
        let keys: Vec<_> = (0..size)
            .map(|i| KeyNode {
                index: i,
                label: format!("k{}", i),
                hand: HandIndex((i % 2) as u8),
                // Use FingerIndex::INDEX (1) for all keys to ensure row-based costs are applied.
                // This is a simplification for testing cost gradients.
                finger: FingerIndex::INDEX,
                row: RowIndex((i / 10) as i8), // 0-9: r0, 10-19: r1, 20-29: r2
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

        let cost_model = mock_cost_model();
        Compiler::compile(&kb, &corpus, &Rubric::default(), &cost_model).unwrap()
    }

    #[test]
    fn test_suggest_swaps_multi_mapped() {
        let ctx = setup_mock_ctx(30);

        // Layout where 'e' (10) is on a very expensive key (index 20, row 2, cost 3.0)
        // and 'Space' (32) is on two keys: index 0 (row 0, cost 1.0) and index 1 (row 0, cost 1.0).
        // We want to swap 'e' to a cheaper spot.
        let mut keys = vec![KeyCode(0); 30];
        for i in 0..30 {
            keys[i] = KeyCode(i as u16);
        }

        keys[20] = KeyCode(10); // 'e' at expensive position (row 2)
        keys[0] = KeyCode(32); // Space at cheap position (row 0)
        keys[1] = KeyCode(32); // Space at cheap position (row 0)

        let layout = Layout::new_unchecked(keys);
        let suggestions = suggest_swaps(&ctx, &layout, true);

        // We expect at least one suggestion involving 'e' (index 20)
        // to be swapped with one of the 'Space' positions or other good positions.
        assert!(!suggestions.is_empty(), "Should suggest improvements");

        // Verify that suggestions include swaps with Space (32)
        let has_space_swap = suggestions
            .iter()
            .any(|s| s.key_a == "32" || s.key_b == "32");
        assert!(
            has_space_swap,
            "Should suggest swapping with Space even if it is multi-mapped"
        );
    }

    fn setup_kb_minimal() -> Keyboard {
        let keys: Vec<KeyNode> = (0..3)
            .map(|i| KeyNode {
                index: i,
                label: format!("k{}", i),
                hand: HandIndex(0),
                finger: FingerIndex::new_unchecked(i as u8),
                x: i as f32,
                ..Default::default()
            })
            .collect();
        Keyboard::new(keys, 0).unwrap()
    }

    #[test]
    fn test_heuristics_swap_suggestion_success() {
        let kb = setup_kb_minimal();
        let mut corpus = Corpus::default();
        corpus.bigrams.push((0, 2, 1000));

        let mut rubric = Rubric::default();
        rubric.travel_lat = 10.0;

        let layout = Layout::new_unchecked(vec![KeyCode(0), KeyCode(1), KeyCode(2)]);
        let engine = EngineFactory::new_generic(&kb, &corpus, &rubric, &mock_cost_model()).unwrap();

        let suggestions = suggest_swaps(engine.context(), &layout, false);
        assert!(
            !suggestions.is_empty(),
            "Should suggest swapping 0 closer to 2"
        );
        assert!(suggestions[0].improvement_pct > 0.0);
    }

    #[test]
    fn test_heuristics_zero_score_early_return() {
        let kb = setup_kb_minimal();
        let corpus = Corpus::default();
        let layout = Layout::new_unchecked(vec![KeyCode(0), KeyCode(1), KeyCode(2)]);
        let engine =
            EngineFactory::new_generic(&kb, &corpus, &Rubric::default(), &mock_cost_model()).unwrap();

        let suggestions = suggest_swaps(engine.context(), &layout, false);
        assert!(
            suggestions.is_empty(),
            "Zero score should return empty suggestions"
        );
    }

    #[test]
    fn test_swap_degradation() {
        let kb = setup_kb_minimal();
        let mut corpus = Corpus::default();
        corpus.bigrams.push((0, 1, 1000));

        let mut rubric = Rubric::default();
        rubric.travel_lat = 10.0;

        let engine = EngineFactory::new_generic(&kb, &corpus, &rubric, &mock_cost_model()).unwrap();

        let layout_keys = vec![KeyCode(0), KeyCode(1), KeyCode(2)];
        let mut pos_map_data = vec![65535u16; 65536];
        pos_map_data[0] = 0;
        pos_map_data[1] = 1;
        pos_map_data[2] = 2;

        let validated = ValidatedLayout::new(&layout_keys, engine.key_count()).unwrap();

        let mut starts = [0u16; 65536];
        let mut counts = [0u8; 65536];
        let mut indices = [0u16; 512];
        let mut current_offsets = [0u8; 65536];
        let mut used_keys_scratch = Vec::with_capacity(layout_keys.len());
        let pm = PosMap::from_scratch(
            &layout_keys,
            engine.key_count(),
            &mut starts,
            &mut counts,
            &mut indices,
            &mut current_offsets,
            &mut used_keys_scratch,
        );

        let delta = calculate_swap_delta(engine.context(), &validated, &pm, 1, 2);

        assert!(delta > 0, "Degrading swap should have positive delta");
    }
}