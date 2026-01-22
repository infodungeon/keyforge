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
//! The computational kernel of `KeyForge`. This crate implements the core
//! scoring logic, biomechanical modeling, and SIMD-accelerated physics engines.

pub mod analysis;
pub mod engines;
pub mod error;
/// Reference ghost models for verification.
pub mod ghost;
pub mod kernel;
pub mod verify;

// --- RE-EXPORTS ---

pub use error::PhysicsError;
pub use kernel::EngineContext;
pub use kernel::compiler::Compiler;
pub use kernel::types::ValidatedLayout;
pub use engines::{EngineCapabilities, EngineFeatures, ScoringEngine};
pub use engines::intel_comet_lake::{IntelEngineConfig, IntelScoringEngine};
pub use engines::arm_neon::{ArmNeonConfig, ArmNeonScoringEngine};
pub use engines::generic::GenericScoringEngine as ScalarScoringEngine;
pub use engines::exact::ExactScoringEngine;
pub use analysis::heuristics::suggest_swaps;
pub use analysis::fingerprint::{Fingerprinter, LayoutIdentity};

// Re-export analysis types from keyforge-model for convenience
pub use keyforge_model::{AnalysisReport, SwapSuggestion};

use keyforge_model::{Corpus, CostModel, Keyboard, Layout, Rubric};
use tracing::instrument;

/// A factory for creating high-performance scoring engines.
#[derive(Debug, Default)]
pub struct EngineFactory;

impl EngineFactory {
    /// Compiles a new **Scalar** (generic) scoring engine.
    ///
    /// # Errors
    /// Returns `PhysicsError` if compilation fails.
    pub fn new_scalar(
        keyboard: &Keyboard,
        corpus: &Corpus,
        rubric: &Rubric,
        cost_model: &CostModel,
    ) -> Result<Box<dyn ScoringEngine>, PhysicsError> {
        let ctx = Compiler::compile(keyboard, corpus, rubric, cost_model)?;
        Ok(Box::new(ScalarScoringEngine::new(ctx)))
    }

    /// Compiles a new **Exact** (Oracle) scoring engine.
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
        Ok(Box::new(ExactScoringEngine::new(keyboard, corpus, rubric, cost_model, ctx)))
    }

    /// Compiles a new generic engine (alias for scalar).
    ///
    /// # Errors
    /// Returns `PhysicsError` if compilation fails.
    pub fn new_generic(
        keyboard: &Keyboard,
        corpus: &Corpus,
        rubric: &Rubric,
        cost_model: &CostModel,
    ) -> Result<Box<dyn ScoringEngine>, PhysicsError> {
        Self::new_scalar(keyboard, corpus, rubric, cost_model)
    }

    /// Compiles a new **Intel AVX2** scoring engine.
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
    let _validated = ValidatedLayout::new(&layout.keys, ctx.key_count)?;
    let engine = ScalarScoringEngine::new(ctx.clone());
    engine.analyze(layout)
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