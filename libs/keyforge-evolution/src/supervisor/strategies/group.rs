use crate::supervisor::traits::{MutationAction, MutationOperator, MutationProposal};
use super::scratch::{KEYS_SCRATCH, POS_MAP_SCRATCH};
use keyforge_model::Layout;
use keyforge_physics::ScoringEngine;
use rand::seq::index::sample;
use rand::Rng;

pub struct GroupMutation {
    pub unlocked_indices: Vec<usize>,
    pub start_temp: f32,
    pub end_temp: f32,
}

impl MutationOperator for GroupMutation {
    #[allow(clippy::cast_possible_truncation)]
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
            let ratio = ((temperature - self.end_temp) / (self.start_temp - self.end_temp))
                .clamp(0.0, 1.0);
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

        // Task-evo-017: Efficient 3-Way Delta
        // 1. Calculate Delta(A, B) on current state
        let d1 = engine.calculate_swap_delta(&layout.keys, pos_map, idx_a, idx_b)?;

        // 2. To calculate Delta(A_at_B, C), we need to simulate state after first swap
        // Instead of full clone, we patch our thread-local scratch keys
        let delta = KEYS_SCRATCH.with(|k_scratch| {
            let mut temp_keys = k_scratch.borrow_mut();

            // Sync with current layout only if size changed or we want a fresh base
            // Optimization: Only copy if needed, but since we revert, it should be fine.
            if temp_keys.len() == layout.keys.len() {
                // Just patch the indices that might have changed from a PREVIOUS failed mutation
                // Actually, safer to just copy since acceptance happens elsewhere.
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
                engine.calculate_swap_delta(&temp_keys, &patched_pos_map, idx_b, idx_c)
            })
        })?;

        Ok(Some(MutationProposal {
            delta: d1 + delta,
            action: MutationAction::GroupSwap(idx_a.into(), idx_b.into(), idx_c.into()),
        }))
    }
}
