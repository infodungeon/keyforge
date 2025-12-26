use keyforge_model::Layout;
use keyforge_physics::ScoringEngine;
use rand::Rng;

#[derive(Debug, Clone, Copy)]
pub enum MutationAction {
    Swap(usize, usize),
    GroupSwap(usize, usize, usize),
}

impl MutationAction {
    #[inline(always)]
    pub fn apply(self, layout: &mut Layout, pos_map: &mut [u8]) {
        match self {
            MutationAction::Swap(a, b) => {
                layout.keys.swap(a, b);
                let code_a = layout.keys[a] as usize;
                let code_b = layout.keys[b] as usize;
                // Update pos_map if within range
                if code_a < pos_map.len() {
                    pos_map[code_a] = a as u8;
                }
                if code_b < pos_map.len() {
                    pos_map[code_b] = b as u8;
                }
            }
            MutationAction::GroupSwap(a, b, c) => {
                // A -> B, B -> C, C -> A
                let temp = layout.keys[c];
                layout.keys[c] = layout.keys[b];
                layout.keys[b] = layout.keys[a];
                layout.keys[a] = temp;

                let code_a = layout.keys[a] as usize;
                let code_b = layout.keys[b] as usize;
                let code_c = layout.keys[c] as usize;

                if code_a < pos_map.len() {
                    pos_map[code_a] = a as u8;
                }
                if code_b < pos_map.len() {
                    pos_map[code_b] = b as u8;
                }
                if code_c < pos_map.len() {
                    pos_map[code_c] = c as u8;
                }
            }
        }
    }
}

/// A proposed change to a layout.
pub struct MutationProposal {
    pub delta: i64,
    /// Enum describing the mutation to apply.
    /// Replaces Box<dyn FnOnce> to avoid heap allocation.
    pub action: MutationAction,
}

/// Defines how to generate a potential layout change.
pub trait MutationOperator {
    fn propose(
        &self,
        engine: &ScoringEngine,
        layout: &Layout,
        pos_map: &[u8],
        rng: &mut impl Rng,
    ) -> Option<MutationProposal>;
}

/// Defines the criteria for accepting a proposed mutation.
pub trait AcceptanceCriteria {
    fn should_accept(&mut self, delta: i64, temperature: f32, rng: &mut impl Rng) -> bool;
}
