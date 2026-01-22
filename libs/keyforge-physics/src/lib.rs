// libs/keyforge-physics/src/lib.rs

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

//! # `KeyForge` Physics
//!
//! The "Physics Engine" of `KeyForge`. This crate implements the core
//! scoring logic, evaluating layouts based on physical constraints
//! and language statistics.

mod analysis;
pub mod engines;
/// Physics-specific error types.
pub mod error;
/// Reference ghost models for verification.
pub mod ghost;
mod kernel;
/// Layout verification and validity scoring.
pub mod verify;

pub use analysis::fingerprint::LayoutIdentity;
pub use keyforge_model::SwapSuggestion;

use analysis::fingerprint::Fingerprinter;
use analysis::heuristics::suggest_swaps;
pub use engines::arm_neon::ArmNeonConfig;
pub use engines::intel_comet_lake::IntelEngineConfig;
use engines::{
    arm_neon::ArmNeonScoringEngine, exact::ExactScoringEngine, generic::GenericScoringEngine,
    intel_comet_lake::IntelScoringEngine,
};
pub use engines::{EngineCapabilities, ScoringEngine};
pub use error::PhysicsError;
use kernel::compiler::Compiler;
use kernel::compute::analyze_layout;
use kernel::types::ValidatedLayout;
pub use kernel::EngineContext;
use keyforge_model::{AnalysisReport, Corpus, CostModel, Keyboard, Layout, Rubric};
use tracing::instrument;

/// Factory for creating scoring engines.
#[derive(Debug)]
pub struct EngineFactory;

impl EngineFactory {
    /// Compiles a new scoring engine from the provided keyboard, corpus, and rubric.
    ///
    /// This uses the default **Generic Optimized** engine.
    ///
    /// # Errors
    ///
    /// Returns a `PhysicsError::Config` if the compilation of the physics kernel fails.
    pub fn new_generic(
        keyboard: &Keyboard,
        corpus: &Corpus,
        rubric: &Rubric,
        cost_model: &CostModel,
    ) -> Result<Box<dyn ScoringEngine>, PhysicsError> {
        let ctx = Compiler::compile(keyboard, corpus, rubric, cost_model)?;
        Ok(Box::new(GenericScoringEngine::new(ctx)))
    }

    /// Compiles a new **Exact (Oracle)** scoring engine.
    ///
    /// This engine is bit-perfect but slow. Use for verification only.
    ///
    /// # Errors
    /// Returns `PhysicsError` if compilation fails.
    pub fn new_exact(
        keyboard: &Keyboard,
        corpus: &Corpus,
        rubric: &Rubric,
        cost_model: &CostModel,
    ) -> Result<Box<dyn ScoringEngine>, PhysicsError> {
        let ctx = Compiler::compile(keyboard, corpus, rubric, cost_model)?;
        Ok(Box::new(ExactScoringEngine::new(
            keyboard.clone(),
            corpus.clone(),
            rubric,
            cost_model,
            ctx,
        )))
    }

    /// Compiles a new **Intel Comet Lake** scoring engine.
    ///
    /// This engine uses AVX2 optimizations and cache-aware access patterns.
    /// It is only safe to use on compatible hardware (checked by caller).
    ///
    /// # Errors
    /// Returns `PhysicsError` if compilation fails.
    pub fn new_intel_comet_lake(
        keyboard: &Keyboard,
        corpus: &Corpus,
        rubric: &Rubric,
        cost_model: &CostModel,
        config: Option<IntelEngineConfig>,
    ) -> Result<Box<dyn ScoringEngine>, PhysicsError> {
        let ctx = Compiler::compile(keyboard, corpus, rubric, cost_model)?;
        Ok(Box::new(IntelScoringEngine::new(ctx, config)))
    }

    /// Compiles a new **ARM NEON** scoring engine.
    ///
    /// This engine uses ARM NEON SIMD optimizations.
    /// It is only safe to use on compatible hardware (checked by caller).
    ///
    /// # Errors
    /// Returns `PhysicsError` if compilation fails.
    pub fn new_arm_neon(
        keyboard: &Keyboard,
        corpus: &Corpus,
        rubric: &Rubric,
        cost_model: &CostModel,
        config: Option<ArmNeonConfig>,
    ) -> Result<Box<dyn ScoringEngine>, PhysicsError> {
        let ctx = Compiler::compile(keyboard, corpus, rubric, cost_model)?;
        Ok(Box::new(ArmNeonScoringEngine::new(ctx, config)))
    }
}

/// Identifies a layout by comparing it to known standards.
#[instrument]
pub fn identify(layout: &Layout) -> Option<LayoutIdentity> {
    Fingerprinter::identify(layout)
}

/// Analyzes a layout and returns a detailed report.
///
/// # Errors
/// Returns `PhysicsError` if the layout is invalid for the context.
pub fn analyze_with_context(
    ctx: &EngineContext,
    layout: &Layout,
) -> Result<AnalysisReport, PhysicsError> {
    let validated = ValidatedLayout::new(&layout.keys, ctx.key_count)?;
    Ok(analyze_layout(ctx, &validated))
}

/// Suggests improvements for the layout.
#[must_use]
pub fn suggest_improvements_with_context(
    ctx: &EngineContext,
    layout: &Layout,
    include_thumbs: bool,
) -> Vec<SwapSuggestion> {
    suggest_swaps(ctx, layout, include_thumbs)
}
    include_thumbs: bool,
) -> Vec<SwapSuggestion> {
    suggest_swaps(ctx, layout, include_thumbs)
}
