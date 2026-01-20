use crate::kernel::EngineContext;
use crate::PhysicsError;
use keyforge_model::{AnalysisReport, Layout, Score, SwapSuggestion};

pub mod exact;
pub mod generic;
pub mod intel_comet_lake;

/// Hardware and performance capabilities of a scoring engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EngineCapabilities {
    /// If true, the engine provides bit-perfect parity with the Oracle.
    pub is_exact: bool,
    /// If true, the engine utilizes AVX2 instructions.
    pub supports_avx2: bool,
    /// If true, the engine uses cache-aware blocking (e.g. for large cost matrices).
    pub supports_blocking: bool,
}

/// Defines a strategy for scoring keyboard layouts.
pub trait ScoringEngine: Send + Sync + std::fmt::Debug {
    /// Returns the name of the scoring engine.
    fn name(&self) -> &'static str;

    /// Returns the capabilities of this engine.
    fn capabilities(&self) -> EngineCapabilities;

    /// Returns the number of keys supported by this engine.
    fn key_count(&self) -> usize;

    /// Calculates the score for a given layout.
    fn score(&self, layout: &Layout) -> Result<Score, PhysicsError>;

    /// Calculates detailed scores (monogram, bigram, trigram) for a layout.
    fn score_detailed(&self, layout: &Layout) -> Result<(i64, i64, i64), PhysicsError>;

    /// Calculates the change in score if two keys were swapped.
    fn calculate_swap_delta(
        &self,
        layout: &Layout,
        pos_map: &[u16],
        idx_a: usize,
        idx_b: usize,
    ) -> Result<i64, PhysicsError>;

    /// Generates a comprehensive ergonomics analysis for a layout.
    fn analyze(&self, layout: &Layout) -> Result<AnalysisReport, PhysicsError>;

    /// Suggests improvements based on the current scoring model.
    fn suggest_improvements(&self, layout: &Layout, include_thumbs: bool) -> Vec<SwapSuggestion>;

    /// Returns a reference to the internal engine context.
    fn context(&self) -> &EngineContext;
}
