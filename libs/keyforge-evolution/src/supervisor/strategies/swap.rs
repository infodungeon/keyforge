use crate::supervisor::traits::{MutationAction, MutationOperator, MutationProposal};
use keyforge_model::Layout;
use keyforge_physics::ScoringEngine;
use rand::Rng;

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
