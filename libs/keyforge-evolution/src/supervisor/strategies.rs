use super::traits::{AcceptanceCriteria, MutationAction, MutationOperator, MutationProposal};
use keyforge_model::Layout;
use keyforge_physics::ScoringEngine;
use rand::Rng;

#[allow(dead_code)]
pub struct SwapMutation {
    pub unlocked_indices: Vec<usize>,
}

impl MutationOperator for SwapMutation {
    fn propose(
        &self,
        engine: &ScoringEngine,
        layout: &Layout,
        pos_map: &[u8],
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
            action: MutationAction::Swap(idx_a, idx_b),
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
        pos_map: &[u8],
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

        if use_swap {
            let i = rng.gen_range(0..len);
            let mut j = rng.gen_range(0..len);
            while i == j {
                j = rng.gen_range(0..len);
            }

            let idx_a = self.unlocked_indices[i];
            let idx_b = self.unlocked_indices[j];

            let delta = engine.calculate_swap_delta(&layout.keys, pos_map, idx_a, idx_b);

            return Some(MutationProposal {
                delta,
                action: MutationAction::Swap(idx_a, idx_b),
            });
        }

        // 3-Way Swap
        let i = rng.gen_range(0..len);
        let mut j = rng.gen_range(0..len);
        while j == i {
            j = rng.gen_range(0..len);
        }
        let mut k = rng.gen_range(0..len);
        while k == i || k == j {
            k = rng.gen_range(0..len);
        }

        let idx_a = self.unlocked_indices[i];
        let idx_b = self.unlocked_indices[j];
        let idx_c = self.unlocked_indices[k];

        // Calculate delta via full score difference (simpler for 3-way)
        let old_score = engine.score_raw(&layout.keys);

        let mut temp_layout = layout.clone();
        let temp = temp_layout.keys[idx_c];
        temp_layout.keys[idx_c] = temp_layout.keys[idx_b];
        temp_layout.keys[idx_b] = temp_layout.keys[idx_a];
        temp_layout.keys[idx_a] = temp;

        let new_score = engine.score_raw(&temp_layout.keys);
        let delta = new_score - old_score;

        Some(MutationProposal {
            delta,
            action: MutationAction::GroupSwap(idx_a, idx_b, idx_c),
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

        // P2 FIX: Updated scaling factor to 1,000,000
        let delta_f = delta as f32 / 1_000_000.0;
        let probability = (-delta_f / temperature).exp();
        rng.gen::<f32>() < probability
    }
}
