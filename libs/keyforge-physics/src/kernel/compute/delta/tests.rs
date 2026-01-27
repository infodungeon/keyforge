// libs/keyforge-physics/src/kernel/compute/delta/tests.rs

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;
    use crate::kernel::compute::{calculate_swap_delta, PhysicsScratch, PosMap};
    use crate::ValidatedLayout;
    use keyforge_model::types::{ColIndex, FingerIndex, HandIndex, KeyCode, RowIndex, Score};
    use keyforge_model::{Corpus, CostModel, KeyNode, Keyboard, Layout, Rubric};
    use proptest::prelude::*;
    use rand::rngs::StdRng;
    use rand::seq::SliceRandom;
    use rand::SeedableRng;
    use std::sync::Arc;

    fn kb_and_layout_strategy() -> impl Strategy<Value = (Keyboard, Vec<KeyCode>)> {
        (10..50usize).prop_flat_map(|count| {
            let kb_strat = prop::collection::vec(
                (
                    -20.0..20.0f32,
                    -20.0..20.0f32,
                    0u8..2,
                    0u8..5,
                    0u8..5, // finger (valid 0..=4)
                    -5i8..5,
                    -10i8..15,
                ),
                count,
            )
            .prop_map(move |keys_data| {
                let keys = keys_data
                    .into_iter()
                    .enumerate()
                    .map(|(i, (x, y, hand, _, finger, row, col))| KeyNode {
                        index: i,
                        label: format!("k{i}"),
                        hand: HandIndex(hand),
                        finger: FingerIndex::new_unchecked(finger),
                        row: RowIndex(row),
                        col: ColIndex(col),
                        x,
                        y,
                        is_home: row == 1,
                        ..Default::default()
                    })
                    .collect();
                Keyboard::new(keys, keyforge_model::types::RowIndex(1), "test".into())
                    .expect("Failed to create keyboard in strategy")
            });

            // Ensure unique keys to avoid invalid layouts
            let layout_strat = prop::collection::hash_set(0u16..255, count)
                .prop_map(|codes| codes.into_iter().map(KeyCode).collect::<Vec<_>>());

            (kb_strat, layout_strat)
        })
    }

    fn corpus_strategy(char_range: std::ops::Range<u16>) -> impl Strategy<Value = Corpus> {
        (
            prop::collection::vec((char_range.clone(), char_range.clone(), 1u32..1000), 0..20),
            prop::collection::vec(
                (
                    char_range.clone(),
                    char_range.clone(),
                    char_range.clone(),
                    1u32..1000,
                ),
                0..20,
            ),
            prop::collection::vec(0u64..1000, 65536),
        )
            .prop_map(|(bigrams, trigrams, char_freqs)| Corpus {
                bigrams: bigrams.into(),
                trigrams: trigrams.into(),
                char_freqs: char_freqs.into(),
                ..Default::default()
            })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn test_delta_validity(
            (kb, mut layout_keys) in kb_and_layout_strategy(),
            cp in corpus_strategy(0..30),
            seed in any::<u64>(),
            swap_idx_1 in 0..100usize,
            swap_idx_2 in 0..100usize
        ) {
            let len = layout_keys.len();
            if len < 2 { return Ok(()); }

            let i = swap_idx_1 % len;
            let j = swap_idx_2 % len;
            if i == j { return Ok(()); }

            let mut rng = StdRng::seed_from_u64(seed);
            layout_keys.shuffle(&mut rng);

            let rubric = Rubric::default();
            // Use a mock cost model instead of fixture to be more resilient
            let mut cm = CostModel::default();
            let mut fingers = std::collections::HashMap::new();
            for f in 0..5 {
                let f_name = match f { 0=>"thumb", 1=>"index", 2=>"middle", 3=>"ring", _=>"pinky" };
                fingers.insert(f_name.to_string(), keyforge_model::cost_model::FingerDefinition::Thumb(std::collections::HashMap::from([("pos_1".into(), 1.0)])));
            }
            cm.models.insert("model_a_row_staggered".into(), keyforge_model::cost_model::ModelDefinition {
                description: "test".into(),
                static_costs: std::collections::HashMap::from([("universal_hand".to_string(), keyforge_model::cost_model::HandDefinition { fingers })]),
            });

            let engine = crate::EngineFactory::new_generic(&crate::EngineCompilationContext {
                keyboard: Arc::new(kb.clone()),
                corpus: Arc::new(cp.clone()),
                rubric: Arc::new(rubric.clone()),
                cost_model: Arc::new(cm.clone()),
                engine_config: keyforge_model::config::EngineConfig::default(),
            })
            .expect("Failed to compile generic engine");

            let layout_for_score = Layout::new_unchecked(layout_keys.clone());
            let score_before = engine.score(&layout_for_score).expect("Failed to score layout before").0;
            if score_before == i64::MAX { return Ok(()); }

            let validated = ValidatedLayout::new(&layout_keys, engine.key_count()).expect("Failed to validate layout");
        let mut scratch = PhysicsScratch::try_new().unwrap();
            let pm = PosMap::from_scratch(
                &layout_keys,
                engine.key_count(),
                &mut scratch.starts,
                &mut scratch.counts,
                scratch.indices.as_mut_slice(),
                &mut scratch.current_offsets,
                &mut scratch.used_keys,
            );

            let delta = calculate_swap_delta(engine.context(), &validated, &pm, i, j).expect("Failed to calculate swap delta");

            layout_keys.swap(i, j);
            let swapped_layout = Layout::new_unchecked(layout_keys.clone());
            let score_after = engine.score(&swapped_layout).expect("Failed to score layout after").0;
            let actual_delta = score_after - score_before;

            prop_assert_eq!(
                actual_delta, delta,
                "Delta mismatch! Actual: {}, Calculated: {}",
                actual_delta, delta
            );
        }
    }

    #[test]
    fn test_calculate_swap_delta_with_modifiers() {
        let keys: Vec<KeyNode> = (0..2)
            .map(|i| {
                let idx_u8 = u8::try_from(i).expect("Index fits in u8");
                let idx_i8 = i8::try_from(i).expect("Index fits in i8");
                KeyNode {
                    index: i,
                    hand: HandIndex(0),
                    finger: FingerIndex::new_unchecked(idx_u8),
                    row: RowIndex(0),
                    col: ColIndex(idx_i8),
                    ..Default::default()
                }
            })
            .collect();
        let kb = Keyboard::new(keys, keyforge_model::types::RowIndex(1), "test".into()).expect("Failed to create keyboard");

        let mut cp = Corpus::default();
        let mut freqs = cp.char_freqs.to_vec();
        freqs[97] = 100; // 'a'
        freqs[98] = 100; // 'b'
        cp.char_freqs = Arc::from(freqs);
        cp.bigrams = Arc::from(vec![(97, 98, 100)]);

        let rubric = Rubric::default();
        let mut cm = CostModel::default();
        let mut fingers = std::collections::HashMap::new();
        for f in 0..5 {
            let f_name = match f {
                0 => "thumb",
                1 => "index",
                2 => "middle",
                3 => "ring",
                _ => "pinky",
            };
            fingers.insert(
                f_name.to_string(),
                keyforge_model::cost_model::FingerDefinition::Thumb(
                    std::collections::HashMap::from([("pos_1".into(), 1.0)]),
                ),
            );
        }
        cm.models.insert(
            "model_a_row_staggered".into(),
            keyforge_model::cost_model::ModelDefinition {
                description: "test".into(),
                static_costs: std::collections::HashMap::from([(
                    "universal_hand".to_string(),
                    keyforge_model::cost_model::HandDefinition { fingers },
                )]),
            },
        );

        let engine = crate::EngineFactory::new_generic(&crate::EngineCompilationContext {
            keyboard: Arc::new(kb.clone()),
            corpus: Arc::new(cp.clone()),
            rubric: Arc::new(rubric.clone()),
            cost_model: Arc::new(cm.clone()),
            engine_config: keyforge_model::config::EngineConfig::default(),
        })
        .expect("Failed to compile generic engine");
        let mut ctx = engine.context().clone();

        let layout_keys = vec![KeyCode(97), KeyCode(98)];
        let validated = ValidatedLayout::new(&layout_keys, engine.key_count())
            .expect("Failed to validate layout");

        let mut scratch = PhysicsScratch::try_new().unwrap();
        let pos_map = PosMap::from_scratch(
            &layout_keys,
            engine.key_count(),
            &mut scratch.starts,
            &mut scratch.counts,
            scratch.indices.as_mut_slice(),
            &mut scratch.current_offsets,
            &mut scratch.used_keys,
        );

        // Force asymmetry in cost matrix so delta is non-zero
        if ctx.geometry.cost_matrix.len() >= 4 {
            let mut m = (*ctx.geometry.cost_matrix).to_vec();
            m[1] = Score(10);
            m[2] = Score(50);
            ctx.geometry.cost_matrix = m.into();
        }

        let mut mod_map = (*ctx.sequence_modifiers).clone();
        mod_map.insert((97, 98), Score(100));
        ctx.sequence_modifiers = Arc::new(mod_map);

        let delta = calculate_swap_delta(&ctx, &validated, &pos_map, 0, 1)
            .expect("Failed to calculate swap delta");
        assert!(delta != 0);
    }
}
