// libs/keyforge-physics/src/kernel/compute/delta/tests.rs

use super::*;
use crate::engines::generic::GenericScoringEngine;
use crate::engines::ScoringEngine;
use crate::kernel::compiler::Compiler;
use crate::kernel::compute::{PhysicsScratch, PosMap};
use crate::kernel::types::ValidatedLayout;
use keyforge_model::types::{ColIndex, FingerIndex, HandIndex, KeyCode, KeyIndex, RowIndex};
use keyforge_model::{Corpus, KeyNode, Keyboard, Layout, Rubric};
use proptest::prelude::*;
use std::sync::Arc;

fn setup_test_ctx(keys: Vec<KeyNode>, bigrams: Vec<(u16, u16, u32)>) -> EngineContext {
    let kb = Keyboard::new(keys, RowIndex::new(0), "test".into()).unwrap();
    let mut corpus = Corpus::default();
    corpus.bigrams = Arc::from(bigrams);
    Compiler::compile(
        &kb,
        &corpus,
        &Rubric::default(),
        &keyforge_model::testing::mock_cost_model(),
    )
    .unwrap()
}

fn setup_basic_kb() -> Vec<KeyNode> {
    vec![
        KeyNode {
            index: KeyIndex::new(0),
            hand: HandIndex::new(0),
            finger: FingerIndex::new(1),
            row: RowIndex::new(0),
            col: ColIndex::new(0),
            ..Default::default()
        },
        KeyNode {
            index: KeyIndex::new(1),
            hand: HandIndex::new(0),
            finger: FingerIndex::new(2),
            row: RowIndex::new(0),
            col: ColIndex::new(1),
            ..Default::default()
        },
    ]
}

proptest! {
    #[test]
    fn test_delta_parity_random(
        hand in 0u8..=1,
        finger in 0u8..=4,
        row in -5i8..=5,
        col in -5i8..=5,
        layout_codes in prop::collection::vec(0u16..255, 2..10)
    ) {
        let keys: Vec<KeyNode> = (0..layout_codes.len())
            .map(|i| KeyNode {
                index: KeyIndex::new(i as u16),
                hand: HandIndex::new(hand),
                finger: FingerIndex::new(finger),
                row: RowIndex::new(row),
                col: ColIndex::new(col),
                ..Default::default()
            })
            .collect();

        let mut bigrams = Vec::new();
        for i in 0..layout_codes.len()-1 {
            bigrams.push((layout_codes[i], layout_codes[i+1], 100));
        }

        let ctx = setup_test_ctx(keys, bigrams);
        let layout = Layout::new_unchecked(layout_codes.iter().map(|&c| KeyCode::new(c)).collect());
        let validated = ValidatedLayout::new(layout.keys(), layout.len()).unwrap();

        let mut scratch = PhysicsScratch::try_new().unwrap();
        let (starts, counts, indices, offsets, used, _, _) = scratch.get_mut_scratch();
        let pm = PosMap::from_scratch(layout.keys(), layout.len(), starts, counts, indices, offsets, used);

        let delta = calculate_swap_delta(&ctx, &validated, &pm, 0, 1).unwrap();

        let mut swapped_keys = layout.keys().to_vec();
        swapped_keys.swap(0, 1);
        let swapped_layout = Layout::new_unchecked(swapped_keys);

        let engine = GenericScoringEngine::new(ctx.clone());
        let score_before = engine.score(&layout).unwrap().raw();
        let score_after = engine.score(&swapped_layout).unwrap().raw();

        assert_eq!(delta, score_after - score_before, "Delta must match score difference exactly");
    }
}

#[keyforge_testing_macros::kf_test]
mod manual_tests {
    use super::*;

    #[test]
    fn test_simple_swap_delta() {
        let keys = setup_basic_kb();
        let bigrams = vec![(97, 98, 100)]; // 'ab'
        let ctx = setup_test_ctx(keys, bigrams);

        let layout = Layout::new_unchecked(vec![KeyCode::new(97), KeyCode::new(98)]);
        let validated = ValidatedLayout::new(layout.keys(), 2).unwrap();

        let mut scratch = PhysicsScratch::try_new().unwrap();
        let (starts, counts, indices, offsets, used, _, _) = scratch.get_mut_scratch();
        let pm = PosMap::from_scratch(layout.keys(), 2, starts, counts, indices, offsets, used);

        // Score before: a at 0, b at 1. cost(0,1) * freq(ab)
        // Score after swap: a at 1, b at 0. cost(1,0) * freq(ab)
        let delta = calculate_swap_delta(&ctx, &validated, &pm, 0, 1).unwrap();

        let mut swapped_keys = layout.keys().to_vec();
        swapped_keys.swap(0, 1);
        let swapped_layout = Layout::new_unchecked(swapped_keys);

        let engine = GenericScoringEngine::new(ctx);
        let score_before = engine.score(&layout).unwrap().raw();
        let score_after = engine.score(&swapped_layout).unwrap().raw();

        assert_eq!(delta, score_after - score_before);
    }
}
