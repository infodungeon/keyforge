use crate::kernel::EngineContext;
use crate::PhysicsError;
use keyforge_model::{AnalysisReport, Layout, Score, SwapSuggestion};

pub mod arm_neon;
pub mod arm_sve;
pub mod exact;
pub mod generic;
pub mod intel_avx512;
pub mod intel_comet_lake;
pub mod wasm_simd;

/// Hardware and performance capabilities of a scoring engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EngineCapabilities {
    /// If true, the engine provides bit-perfect parity with the Oracle.
    pub is_exact: bool,
    /// Hardware acceleration and optimization features.
    pub features: EngineFeatures,
}

use bitflags::bitflags;

bitflags! {
    /// Specific optimization features supported by an engine.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct EngineFeatures: u32 {
        /// No special optimization features.
        const NONE = 0;
        /// The engine utilizes AVX2 instructions.
        const AVX2 = 1 << 0;
        /// The engine utilizes AVX-512 instructions.
        const AVX512 = 1 << 1;
        /// The engine utilizes ARM NEON instructions.
        const NEON = 1 << 2;
        /// The engine utilizes ARM SVE/SVE2 instructions.
        const SVE = 1 << 3;
        /// The engine utilizes WebAssembly SIMD instructions.
        const WASM_SIMD = 1 << 4;
        /// The engine uses cache-aware blocking (e.g. for large cost matrices).
        const BLOCKING = 1 << 5;
    }
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
    ///
    /// # Errors
    /// Returns `PhysicsError` if scoring fails or overflows.
    fn score(&self, layout: &Layout) -> Result<Score, PhysicsError>;

    /// Calculates the score for a given layout using provided scratch space.
    ///
    /// # Errors
    /// Returns `PhysicsError` if scoring fails or overflows.
    fn score_with_scratch(
        &self,
        layout: &Layout,
        scratch: &mut crate::kernel::compute::PhysicsScratch,
    ) -> Result<Score, PhysicsError>;

    /// Calculates detailed scores (monogram, bigram, trigram) for a layout.
    ///
    /// # Errors
    /// Returns `PhysicsError` if scoring fails or overflows.
    fn score_detailed(&self, layout: &Layout) -> Result<(i64, i64, i64), PhysicsError>;

    /// Calculates the change in score if two keys were swapped.
    ///
    /// # Errors
    /// Returns `PhysicsError` if delta calculation fails.
    fn calculate_swap_delta(
        &self,
        layout: &Layout,
        pos_map: &[keyforge_model::types::KeyIndex],
        idx_a: usize,
        idx_b: usize,
    ) -> Result<i64, PhysicsError>;

    /// Generates a comprehensive ergonomics analysis for a layout.
    ///
    /// # Errors
    /// Returns `PhysicsError` if analysis fails.
    fn analyze(&self, layout: &Layout) -> Result<AnalysisReport, PhysicsError>;

    /// Suggests improvements based on the current scoring model.
    fn suggest_improvements(&self, layout: &Layout, include_thumbs: bool) -> Vec<SwapSuggestion>;

    /// Returns a reference to the internal engine context.
    fn context(&self) -> &EngineContext;
}
