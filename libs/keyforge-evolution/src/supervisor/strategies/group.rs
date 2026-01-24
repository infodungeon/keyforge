use super::scratch::{KEYS_SCRATCH, POS_MAP_SCRATCH};
use crate::supervisor::traits::{MutationAction, MutationOperator, MutationProposal};
use keyforge_model::Layout;
use keyforge_physics::ScoringEngine;
use rand::seq::index::sample;
use rand::Rng;

#[derive(Debug)]
pub struct GroupMutation {
    pub unlocked_indices: Vec<usize>,
    pub start_temp: f32,
    pub end_temp: f32,
}

impl MutationOperator for GroupMutation {
    #[allow(clippy::cast_possible_truncation)]
    fn propose(
        &self,
        engine: &dyn ScoringEngine,
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
            let ratio =
                ((temperature - self.end_temp) / (self.start_temp - self.end_temp)).clamp(0.0, 1.0);
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
            let delta = engine.calculate_swap_delta(layout, pos_map, idx_a, idx_b)?;

            return Ok(Some(MutationProposal {
                delta,
                action: MutationAction::Swap(idx_a.into(), idx_b.into()),
            }));
        }

        // 3-Way Swap (A->B, B->C, C->A)
        let idx_c = self.unlocked_indices[indices.index(2)];

        // Task-evo-017: Efficient 3-Way Delta
        // 1. Calculate Delta(A, B) on current state
        let d1 = engine.calculate_swap_delta(layout, pos_map, idx_a, idx_b)?;

        // 2. To calculate Delta(A_at_B, C), we need to simulate state after first swap
        // Instead of full clone, we patch our thread-local scratch keys
        let delta = KEYS_SCRATCH.with(|k_scratch| {
            let mut temp_keys = k_scratch.borrow_mut();

            // Sync with current layout only if size changed or we want a fresh base
            // Optimization: Only copy if needed, but since we revert, it should be fine.
            if temp_keys.len() == layout.keys.len() {
                // Just patch the indices that might have changed from a PREVIOUS failed mutation
                // Actually, safer to just copy since acceptance elsewhere.
                // But we can use copy_from_slice which is very fast.
                temp_keys.copy_from_slice(&layout.keys);
            } else {
                temp_keys.clear();
                temp_keys.extend_from_slice(&layout.keys);
            }

            // Apply virtual swap A <-> B in scratch
            temp_keys.swap(idx_a, idx_b);

            // Update virtual pos_map
            POS_MAP_SCRATCH.with(|pm_scratch| {
                let mut patched_pos_map = pm_scratch.borrow_mut();
                if patched_pos_map.len() < pos_map.len() {
                    patched_pos_map.resize(pos_map.len(), 65535);
                }
                patched_pos_map[..pos_map.len()].copy_from_slice(pos_map);

                let code_a = layout.keys[idx_a];
                let code_b = layout.keys[idx_b];
                if (code_a.0 as usize) < patched_pos_map.len() {
                    patched_pos_map[code_a.0 as usize] = idx_b as u16;
                }
                if (code_b.0 as usize) < patched_pos_map.len() {
                    patched_pos_map[code_b.0 as usize] = idx_a as u16;
                }

                // Calculate second swap delta (A which is at B, with C)
                let temp_layout = Layout::new_unchecked(temp_keys.clone());
                engine.calculate_swap_delta(&temp_layout, &patched_pos_map, idx_b, idx_c)
            })
        })?;

        Ok(Some(MutationProposal {
            delta: d1 + delta,
            action: MutationAction::GroupSwap(idx_a.into(), idx_b.into(), idx_c.into()),
        }))
    }
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;
    use crate::supervisor::state::SearchState;
    use keyforge_model::types::{ColIndex, FingerIndex, HandIndex, KeyCode, RowIndex};
    use keyforge_model::{Corpus, CostModel, KeyNode, Keyboard, Layout, Rubric};
    use keyforge_physics::EngineFactory;
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

    fn setup_engine(size: usize) -> Box<dyn ScoringEngine> {
        let keys: Vec<_> = (0..size)
            .map(|i| KeyNode {
                index: i,
                label: format!("k{i}"),
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
        let kb = Keyboard::new(keys, 1, "test".into()).unwrap();
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
        EngineFactory::new_generic(keyforge_physics::EngineCompilationContext {
            keyboard: &kb,
            corpus: &corpus,
            rubric: &Rubric::default(),
            cost_model: &cost_model,
        })
        .unwrap()
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
                    let score_before = engine.score(state.layout())?.0;
                    let mutation = GroupMutation { unlocked_indices: (0..size).collect(), start_temp: 100.0, end_temp: 0.1 };
                    let mut rng_mutation = Xoshiro256PlusPlus::seed_from_u64(seed);
                    if let Ok(Some(proposal)) = MutationOperator::propose(
                        &mutation,
                        engine.as_ref(),
                        state.layout(),
                        state.pos_map(),
                        &mut rng_mutation,
                        1.0
                    ) {
                        state.apply_mutation(proposal.action);
                        let score_after = engine.score(state.layout())?.0;
                        let actual_delta = score_after - score_before;

                        // Allow minor drift for generic engine deltas
                        let drift = (proposal.delta - actual_delta).abs();
                        prop_assert!(drift <= 1000, "Mutation delta {} vs actual {} drift {} exceeds limit", proposal.delta, actual_delta, drift);
                    }
                }
            }

    #[test]
    fn test_group_mutation_edge_cases() {
        let size = 5;
        let engine = setup_engine(size);
        let layout = Layout::new_unchecked((0..size as u16).map(KeyCode).collect());
        let pos_map = vec![0, 1, 2, 3, 4];
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        // 1. Too few unlocked indices
        let mutation = GroupMutation {
            unlocked_indices: vec![0],
            start_temp: 100.0,
            end_temp: 0.1,
        };
        let res = mutation
            .propose(engine.as_ref(), &layout, &pos_map, &mut rng, 1.0)
            .unwrap();
        assert!(res.is_none());

        // 2. High temp (low p_swap -> more 3-way)
        let mutation = GroupMutation {
            unlocked_indices: (0..size).collect(),
            start_temp: 100.0,
            end_temp: 0.1,
        };
        let _ = mutation
            .propose(engine.as_ref(), &layout, &pos_map, &mut rng, 100.0)
            .unwrap();

        // 3. Low temp (high p_swap -> more single swap)
        let _ = mutation
            .propose(engine.as_ref(), &layout, &pos_map, &mut rng, 0.1)
            .unwrap();

        // 4. Equal start/end temp (p_swap = 0.5)
        let mutation_eq = GroupMutation {
            unlocked_indices: (0..size).collect(),
            start_temp: 1.0,
            end_temp: 1.0,
        };
        let _ = mutation_eq
            .propose(engine.as_ref(), &layout, &pos_map, &mut rng, 1.0)
            .unwrap();
    }
}
