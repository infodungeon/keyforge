pub mod annealing;
pub mod group;
pub mod scratch;

pub use annealing::CoolingAnnealing;
pub use group::GroupMutation;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::supervisor::state::SearchState;
    use keyforge_model::types::{ColIndex, FingerIndex, HandIndex, KeyCode, RowIndex};
    use keyforge_model::{Corpus, CostModel, KeyNode, Keyboard, Layout, Rubric};
    use keyforge_physics::ScoringEngine;
    use proptest::prelude::*;
    use rand::seq::SliceRandom;
    use rand::SeedableRng;
    use rand_xoshiro::Xoshiro256PlusPlus;

    fn mock_cost_model() -> CostModel {
        let json = r#"{
            "meta": { "version": "2.0", "description": "Test", "unit": "pts" },
            "models": {
                "model_a_row_staggered": {
                    "description": "Test Model",
                    "static_costs": {
                        "universal_hand": {
                            "thumb": { "pos_1": 100.0 },
                            "index": { "base": { "r0": 100.0 } },
                            "middle": { "base": { "r0": 100.0 } },
                            "ring": { "base": { "r0": 100.0 } },
                            "pinky": { "base": { "r0": 100.0 } }
                        }
                    }
                }
            },
            "dynamic_rules": { "sequence_modifiers": {}, "penalties": {}, "constraints": {} }
        }"#;
        serde_json::from_str(json).unwrap()
    }

    fn setup_engine(size: usize) -> ScoringEngine {
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
                is_home: false,
                ..Default::default()
            })
            .collect();
        let kb = Keyboard::new(keys, 1).unwrap();
        let mut corpus = Corpus::default();
        for i in 0..size {
            corpus.char_freqs[i] = 100;
            for j in 0..size {
                if i != j {
                    corpus.bigrams.push((i as u16, j as u16, 10));
                }
            }
        }
        let cost_model = mock_cost_model();
        ScoringEngine::new(&kb, &corpus, &Rubric::default(), &cost_model).unwrap()
    }

    proptest! {
        #[test]
        fn test_group_mutation_delta_oracle(
            seed in any::<u64>(),
            layout_seed in any::<u64>()
        ) {
            let size = 10;
            let engine = setup_engine(size);
            let mut keys: Vec<KeyCode> = (0..size as u16).map(KeyCode).collect();
            let mut rng_layout = Xoshiro256PlusPlus::seed_from_u64(layout_seed);
            keys.shuffle(&mut rng_layout);
            let layout = Layout::new_unchecked(keys);
            let mut state = SearchState::new(layout, 0, 1.0).unwrap();
            let score_before = engine.score_raw(&state.layout().keys).unwrap();
            let mutation = GroupMutation { unlocked_indices: (0..size).collect(), start_temp: 100.0, end_temp: 0.1 };
            let mut rng_mutation = Xoshiro256PlusPlus::seed_from_u64(seed);
            // Pass temp=1.0
            if let Ok(Some(proposal)) = crate::supervisor::traits::MutationOperator::propose(
                &mutation,
                &engine,
                state.layout(),
                state.pos_map(),
                &mut rng_mutation,
                1.0
            ) {
                state.apply_mutation(proposal.action);
                let score_after = engine.score_raw(&state.layout().keys).unwrap();
                let actual_delta = score_after - score_before;
                prop_assert_eq!(proposal.delta, actual_delta);
            }
        }
    }
}
