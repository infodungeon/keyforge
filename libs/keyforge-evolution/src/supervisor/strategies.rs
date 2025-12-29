use super::traits::{AcceptanceCriteria, MutationAction, MutationOperator, MutationProposal};
use keyforge_model::Layout;
use keyforge_physics::ScoringEngine;
use keyforge_protocol::constants::SCORE_SCALE;
use rand::Rng;
use rand::seq::index::sample;

#[allow(dead_code)]
pub struct SwapMutation {
    pub unlocked_indices: Vec<usize>,
}

impl MutationOperator for SwapMutation {
    fn propose(
        &self,
        engine: &ScoringEngine,
        layout: &Layout,
        pos_map: &[u16],
        rng: &mut impl Rng,
    ) -> Option<MutationProposal> {
        let len = self.unlocked_indices.len();
        if len < 2 {
            return None;
        }

        let i = rng.gen_range(0..len);
        let mut j = rng.gen_range(0..len);

        if i == j {
            j = (j + 1) % len;
        }

        let idx_a = self.unlocked_indices[i];
        let idx_b = self.unlocked_indices[j];

        let delta = engine.calculate_swap_delta(&layout.keys, pos_map, idx_a, idx_b);

        Some(MutationProposal {
            delta,
            action: MutationAction::Swap(idx_a.into(), idx_b.into()),
        })
    }
}

pub struct GroupMutation {
    pub unlocked_indices: Vec<usize>,
}

impl MutationOperator for GroupMutation {
    fn propose(
        &self,
        engine: &ScoringEngine,
        layout: &Layout,
        pos_map: &[u16],
        rng: &mut impl Rng,
    ) -> Option<MutationProposal> {
        let len = self.unlocked_indices.len();
        if len < 2 {
            return None;
        }

        // Adaptive Strategy:
        // If < 3 keys, must use 2-way swap.
        // If >= 3 keys, 50% chance of 2-way, 50% chance of 3-way.
        let use_swap = len < 3 || rng.gen_bool(0.5);
        let sample_size = if use_swap { 2 } else { 3 };

        let indices = sample(rng, len, sample_size);
        let idx_a = self.unlocked_indices[indices.index(0)];
        let idx_b = self.unlocked_indices[indices.index(1)];

        if use_swap {
            let delta = engine.calculate_swap_delta(&layout.keys, pos_map, idx_a, idx_b);

            return Some(MutationProposal {
                delta,
                action: MutationAction::Swap(idx_a.into(), idx_b.into()),
            });
        }

        // 3-Way Swap
        let idx_c = self.unlocked_indices[indices.index(2)];

        // Decomposed Delta Calculation (O(N))
        // 1. Swap A <-> B
        let d1 = engine.calculate_swap_delta(&layout.keys, pos_map, idx_a, idx_b);

        // 2. Simulate state after first swap
        let mut temp_pos_map = pos_map.to_vec();
        let code_a = layout.keys[idx_a];
        let code_b = layout.keys[idx_b];
        if (code_a.0 as usize) < temp_pos_map.len() { temp_pos_map[code_a.0 as usize] = idx_b as u16; }
        if (code_b.0 as usize) < temp_pos_map.len() { temp_pos_map[code_b.0 as usize] = idx_a as u16; }

        let mut temp_keys = layout.keys.clone();
        temp_keys.swap(idx_a, idx_b);

        // 3. Swap A (now at idx_b) <-> C (at idx_c) ?
        // Indices: a, b, c.
        // Start: [A, B, C]
        // Swap(a, b): [B, A, C]
        // Swap(a, c): [C, A, B] -> Correct rotation.
        let d2 = engine.calculate_swap_delta(&temp_keys, &temp_pos_map, idx_a, idx_c);

        Some(MutationProposal {
            delta: d1 + d2,
            action: MutationAction::GroupSwap(idx_a.into(), idx_b.into(), idx_c.into()),
        })
    }
}

pub struct CoolingAnnealing;

impl AcceptanceCriteria for CoolingAnnealing {
    fn should_accept(&mut self, delta: i64, temperature: f32, rng: &mut impl Rng) -> bool {
        if delta <= 0 {
            return true;
        }

        if temperature < 1e-6 {
            return false;
        }

        // FIX: Use SCORE_SCALE instead of hardcoded 1,000,000.0
        let delta_f = delta as f32 / SCORE_SCALE;
        // INVARIANT: kani::assume(temperature > 0.0);
        let probability = (-delta_f / temperature).exp();
        rng.gen::<f32>() < probability
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::supervisor::state::SearchState;
    use keyforge_model::{Corpus, KeyNode, Keyboard, Layout, Rubric};
    use keyforge_physics::ScoringEngine;
    use proptest::prelude::*;
    use rand::SeedableRng;
    use rand_xoshiro::Xoshiro256PlusPlus;

    use keyforge_model::types::{HandIndex, FingerIndex, RowIndex, ColIndex, KeyCode};

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
        ScoringEngine::new(&kb, &corpus, &Rubric::default(), &[]).unwrap()
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
            use rand::seq::SliceRandom;
            keys.shuffle(&mut rng_layout);
            let layout = Layout::new_unchecked(keys);
            let mut state = SearchState::new(layout, 0, 1.0);
            let score_before = engine.score_raw(&state.layout().keys);
            let mutation = GroupMutation { unlocked_indices: (0..size).collect() };
            let mut rng_mutation = Xoshiro256PlusPlus::seed_from_u64(seed);
            if let Some(proposal) = mutation.propose(&engine, state.layout(), state.pos_map(), &mut rng_mutation) {
                state.apply_mutation(proposal.action);
                let score_after = engine.score_raw(&state.layout().keys);
                let actual_delta = score_after - score_before;
                prop_assert_eq!(proposal.delta, actual_delta);
            }
        }
    }
}
