// libs/keyforge-evolution/src/supervisor/strategies.rs

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

use super::traits::{AcceptanceCriteria, MutationAction, MutationOperator, MutationProposal};
use keyforge_model::{Layout, KeyCode};
use keyforge_physics::ScoringEngine;
use keyforge_model::constants::{SCORE_SCALE, ANNEALING_MIN_TEMP};
use rand::Rng;
use rand::seq::index::sample;
use std::cell::RefCell;

thread_local! {
    static POS_MAP_SCRATCH: RefCell<Vec<u16>> = RefCell::new(vec![0u16; 65536]);
    static KEYS_SCRATCH: RefCell<Vec<KeyCode>> = RefCell::new(Vec::with_capacity(128));
}

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
        _temperature: f32,
    ) -> Result<Option<MutationProposal>, crate::errors::EvolutionError> {
        let len = self.unlocked_indices.len();
        if len < 2 {
            return Ok(None);
        }

        let i = rng.random_range(0..len);
        let mut j = rng.random_range(0..len);

        if i == j {
            j = (j + 1) % len;
        }

        let idx_a = self.unlocked_indices[i];
        let idx_b = self.unlocked_indices[j];

        let delta = engine.calculate_swap_delta(&layout.keys, pos_map, idx_a, idx_b)?;

        Ok(Some(MutationProposal {
            delta,
            action: MutationAction::Swap(idx_a.into(), idx_b.into()),
        }))
    }
}

pub struct GroupMutation {
    pub unlocked_indices: Vec<usize>,
    pub start_temp: f32,
    pub end_temp: f32,
}

impl MutationOperator for GroupMutation {
    fn propose(
        &self,
        engine: &ScoringEngine,
        layout: &Layout,
        pos_map: &[u16],
        rng: &mut impl Rng,
        temperature: f32,
    ) -> Result<Option<MutationProposal>, crate::errors::EvolutionError> {
        let len = self.unlocked_indices.len();
        if len < 2 {
            return Ok(None);
        }

        // Adaptive Strategy:
        // High temp -> High chaos (more group swaps/3-way)
        // Low temp -> Low chaos (more single swaps)
        // ratio 1.0 = start (high), ratio 0.0 = end (low)
        let p_swap = if (self.start_temp - self.end_temp).abs() < f32::EPSILON {
             0.5
        } else {
             let ratio = ((temperature - self.end_temp) / (self.start_temp - self.end_temp)).clamp(0.0, 1.0);
             // At ratio 1.0 (start): p_swap = 0.2 (20% swap, 80% 3-way)
             // At ratio 0.0 (end): p_swap = 0.8 (80% swap, 20% 3-way)
             0.8 - 0.6 * ratio
        };

        let use_swap = len < 3 || rng.random_bool(p_swap.into());
        let sample_size = if use_swap { 2 } else { 3 };

        let indices = sample(rng, len, sample_size);
        let idx_a = self.unlocked_indices[indices.index(0)];
        let idx_b = self.unlocked_indices[indices.index(1)];

        if use_swap {
            let delta = engine.calculate_swap_delta(&layout.keys, pos_map, idx_a, idx_b)?;

            return Ok(Some(MutationProposal {
                delta,
                action: MutationAction::Swap(idx_a.into(), idx_b.into()),
            }));
        }

        // 3-Way Swap (A->B, B->C, C->A)
        let idx_c = self.unlocked_indices[indices.index(2)];

        // Decomposed Delta Calculation (Zero Allocation)
        // 1. Swap A <-> B
        let d1 = engine.calculate_swap_delta(&layout.keys, pos_map, idx_a, idx_b)?;

        // 2. Simulate virtual state after first swap without cloning
        let delta = POS_MAP_SCRATCH.with(|scratch| {
            let mut patched_vec = scratch.borrow_mut();
            // Ensure size matches pos_map
            if patched_vec.len() < pos_map.len() {
                patched_vec.resize(pos_map.len(), 65535);
            }
            
            let patched_pos_map = &mut patched_vec[..pos_map.len()];
            patched_pos_map.copy_from_slice(pos_map);
            
            let code_a = layout.keys[idx_a];
            let code_b = layout.keys[idx_b];
            
            // Update virtual pos_map
            if (code_a.0 as usize) < patched_pos_map.len() { patched_pos_map[code_a.0 as usize] = idx_b as u16; }
            if (code_b.0 as usize) < patched_pos_map.len() { patched_pos_map[code_b.0 as usize] = idx_a as u16; }

            // Swap A (now at idx_b) with C (at idx_c)
            KEYS_SCRATCH.with(|k_scratch| {
                let mut temp_keys = k_scratch.borrow_mut();
                temp_keys.clear();
                temp_keys.extend_from_slice(&layout.keys);
                temp_keys.swap(idx_a, idx_b);

                engine.calculate_swap_delta(&temp_keys, patched_pos_map, idx_a, idx_c)
            })
        })?;

        Ok(Some(MutationProposal {
            delta: d1 + delta,
            action: MutationAction::GroupSwap(idx_a.into(), idx_b.into(), idx_c.into()),
        }))
    }
}

pub struct CoolingAnnealing;

impl AcceptanceCriteria for CoolingAnnealing {
    fn should_accept(&mut self, delta: i64, temperature: f32, rng: &mut impl Rng) -> bool {
        if delta <= 0 {
            return true;
        }

        if temperature < ANNEALING_MIN_TEMP {
            return false;
        }

        // FIX: Use SCORE_SCALE instead of hardcoded 1,000,000.0
        let delta_f = delta as f32 / SCORE_SCALE;
        // INVARIANT: kani::assume(temperature > 0.0);
        let probability = (-delta_f / temperature).exp();
        rng.random::<f32>() < probability
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
        let cost_matrix = vec![];
        ScoringEngine::new(&kb, &corpus, &Rubric::default(), &cost_matrix).unwrap()
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
            let mut state = SearchState::new(layout, 0, 1.0).unwrap();
            let score_before = engine.score_raw(&state.layout().keys).unwrap();
            let mutation = GroupMutation { unlocked_indices: (0..size).collect(), start_temp: 100.0, end_temp: 0.1 };
            let mut rng_mutation = Xoshiro256PlusPlus::seed_from_u64(seed);
            // Pass temp=1.0
            if let Ok(Some(proposal)) = mutation.propose(&engine, state.layout(), state.pos_map(), &mut rng_mutation, 1.0) {
                state.apply_mutation(proposal.action);
                let score_after = engine.score_raw(&state.layout().keys).unwrap();
                let actual_delta = score_after - score_before;
                prop_assert_eq!(proposal.delta, actual_delta);
            }
        }
    }
}