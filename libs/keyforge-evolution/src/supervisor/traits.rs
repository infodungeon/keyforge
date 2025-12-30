use keyforge_model::{KeyIndex, Layout};
use keyforge_physics::ScoringEngine;
use rand::Rng;
use std::time::{Duration, Instant};

/// Abstracts time for deterministic testing (Functional Purity).
pub trait TimeKeeper {
    fn now(&self) -> Instant;
    fn elapsed(&self, start: Instant) -> Duration;
}

/// Default implementation using system time.
#[derive(Debug, Default, Clone, Copy)]
pub struct RealTimeKeeper;

impl TimeKeeper for RealTimeKeeper {
    fn now(&self) -> Instant {
        Instant::now()
    }
    
    fn elapsed(&self, start: Instant) -> Duration {
        start.elapsed()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MutationAction {
    Swap(KeyIndex, KeyIndex),
    GroupSwap(KeyIndex, KeyIndex, KeyIndex),
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
        pos_map: &[u16],
        rng: &mut impl Rng,
    ) -> Result<Option<MutationProposal>, crate::errors::EvolutionError>;
}

/// Defines the criteria for accepting a proposed mutation.
pub trait AcceptanceCriteria {
    fn should_accept(&mut self, delta: i64, temperature: f32, rng: &mut impl Rng) -> bool;
}
