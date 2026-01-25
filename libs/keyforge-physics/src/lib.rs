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

pub use analysis::fingerprint::{Fingerprinter, LayoutIdentity};
pub use analysis::heuristics::suggest_swaps;
pub use engines::arm_neon::{ArmNeonConfig, ArmNeonScoringEngine};
pub use engines::arm_sve::{ArmSveConfig, ArmSveScoringEngine};
pub use engines::exact::ExactScoringEngine;
pub use engines::generic::GenericScoringEngine as ScalarScoringEngine;
pub use engines::intel_avx512::{Avx512Config, Avx512ScoringEngine};
pub use engines::intel_comet_lake::{IntelEngineConfig, IntelScoringEngine};
pub use engines::wasm_simd::{WasmSimdConfig, WasmSimdScoringEngine};
pub use engines::{EngineCapabilities, EngineFeatures, ScoringEngine};
pub use error::PhysicsError;
pub use kernel::compiler::Compiler;
pub use kernel::types::ValidatedLayout;
pub use kernel::EngineContext;

// Re-export analysis types from keyforge-model for convenience
pub use keyforge_model::{AnalysisReport, SwapSuggestion};

use keyforge_model::{Corpus, CostModel, Keyboard, Layout, Rubric};
use std::sync::Arc;
use tracing::instrument;

/// Context required to compile a scoring engine.
/// Refactored to use Arc to eliminate unnecessary clones across the stack.
#[derive(Debug, Clone)]
pub struct EngineCompilationContext {
    /// Physical keyboard definition.
    pub keyboard: Arc<Keyboard>,
    /// Language frequency data.
    pub corpus: Arc<Corpus>,
    /// Scoring weights and penalties.
    pub rubric: Arc<Rubric>,
    /// Biomechanical cost model.
    pub cost_model: Arc<CostModel>,
}

/// A factory for creating high-performance scoring engines.
#[derive(Debug, Default)]
pub struct EngineFactory;

impl EngineFactory {
    /// Compiles a new **Scalar** (generic) scoring engine.
    ///
    /// # Errors
    /// Returns `PhysicsError` if compilation fails.
    pub fn new_scalar(
        ctx: &EngineCompilationContext,
    ) -> Result<Box<dyn ScoringEngine>, PhysicsError> {
        let compiled = Compiler::compile(&ctx.keyboard, &ctx.corpus, &ctx.rubric, &ctx.cost_model)?;
        Ok(Box::new(ScalarScoringEngine::new(compiled)))
    }

    /// Compiles a new **Exact** (Oracle) scoring engine.
    ///
    /// # Errors
    /// Returns `PhysicsError` if compilation fails.
    pub fn new_exact(
        ctx: &EngineCompilationContext,
    ) -> Result<Box<dyn ScoringEngine>, PhysicsError> {
        let compiled = Compiler::compile(&ctx.keyboard, &ctx.corpus, &ctx.rubric, &ctx.cost_model)?;
        Ok(Box::new(ExactScoringEngine::new(
            ctx.keyboard.clone(),
            ctx.corpus.clone(),
            &ctx.rubric,
            &ctx.cost_model,
            compiled,
        )))
    }

    /// Compiles a new generic engine (alias for scalar).
    ///
    /// # Errors
    /// Returns `PhysicsError` if compilation fails.
    pub fn new_generic(
        ctx: &EngineCompilationContext,
    ) -> Result<Box<dyn ScoringEngine>, PhysicsError> {
        Self::new_scalar(ctx)
    }

    /// Compiles the most optimized engine available for the current hardware.
    ///
    /// # Errors
    /// Returns `PhysicsError` if compilation fails.
    pub fn new_optimized(
        ctx: &EngineCompilationContext,
    ) -> Result<Box<dyn ScoringEngine>, PhysicsError> {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            if is_x86_feature_detected!("avx512f")
                && is_x86_feature_detected!("avx512dq")
                && is_x86_feature_detected!("avx512bw")
            {
                return Self::new_intel_avx512(ctx, None);
            }
            if is_x86_feature_detected!("avx2") {
                return Self::new_intel_comet_lake(ctx, None);
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            if std::arch::is_aarch64_feature_detected!("sve") {
                return Self::new_arm_sve(ctx, None);
            }
            return Self::new_arm_neon(ctx, None);
        }

        #[cfg(target_arch = "wasm32")]
        {
            return Self::new_wasm_simd(ctx, None);
        }

        #[cfg(not(any(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_arch = "aarch64",
            target_arch = "wasm32"
        )))]
        {
            Self::new_scalar(ctx)
        }
        #[cfg(any(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_arch = "aarch64",
            target_arch = "wasm32"
        ))]
        {
            // Fallback for x86 if AVX-512/AVX2 is not detected at runtime,
            // or if we somehow pass through the other arches.
            Self::new_scalar(ctx)
        }
    }

    /// Compiles a new **WASM SIMD** scoring engine.
    ///
    /// # Errors
    /// Returns `PhysicsError` if compilation fails.
    pub fn new_wasm_simd(
        ctx: &EngineCompilationContext,
        config: Option<WasmSimdConfig>,
    ) -> Result<Box<dyn ScoringEngine>, PhysicsError> {
        let compiled = Compiler::compile(&ctx.keyboard, &ctx.corpus, &ctx.rubric, &ctx.cost_model)?;
        Ok(Box::new(WasmSimdScoringEngine::new(compiled, config)))
    }

    /// Compiles a new **Intel AVX-512** scoring engine.
    ///
    /// # Errors
    /// Returns `PhysicsError` if compilation fails.
    pub fn new_intel_avx512(
        ctx: &EngineCompilationContext,
        config: Option<Avx512Config>,
    ) -> Result<Box<dyn ScoringEngine>, PhysicsError> {
        let compiled = Compiler::compile(&ctx.keyboard, &ctx.corpus, &ctx.rubric, &ctx.cost_model)?;
        Ok(Box::new(Avx512ScoringEngine::new(compiled, config)))
    }

    /// Compiles a new **Intel AVX2** scoring engine.
    ///
    /// # Errors
    /// Returns `PhysicsError` if compilation fails.
    pub fn new_intel_comet_lake(
        ctx: &EngineCompilationContext,
        config: Option<IntelEngineConfig>,
    ) -> Result<Box<dyn ScoringEngine>, PhysicsError> {
        let compiled = Compiler::compile(&ctx.keyboard, &ctx.corpus, &ctx.rubric, &ctx.cost_model)?;
        Ok(Box::new(IntelScoringEngine::new(compiled, config)))
    }

    /// Compiles a new **ARM SVE** scoring engine.
    ///
    /// # Errors
    /// Returns `PhysicsError` if compilation fails.
    pub fn new_arm_sve(
        ctx: &EngineCompilationContext,
        config: Option<ArmSveConfig>,
    ) -> Result<Box<dyn ScoringEngine>, PhysicsError> {
        let compiled = Compiler::compile(&ctx.keyboard, &ctx.corpus, &ctx.rubric, &ctx.cost_model)?;
        Ok(Box::new(ArmSveScoringEngine::new(compiled, config)))
    }

    /// Compiles a new **ARM NEON** scoring engine.
    ///
    /// # Errors
    /// Returns `PhysicsError` if compilation fails.
    pub fn new_arm_neon(
        ctx: &EngineCompilationContext,
        config: Option<ArmNeonConfig>,
    ) -> Result<Box<dyn ScoringEngine>, PhysicsError> {
        let compiled = Compiler::compile(&ctx.keyboard, &ctx.corpus, &ctx.rubric, &ctx.cost_model)?;
        Ok(Box::new(ArmNeonScoringEngine::new(compiled, config)))
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
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("avx512f")
            && is_x86_feature_detected!("avx512dq")
            && is_x86_feature_detected!("avx512bw")
        {
            let engine = Avx512ScoringEngine::new(ctx.clone(), None);
            return engine.analyze(layout);
        }
        if is_x86_feature_detected!("avx2") {
            let engine = IntelScoringEngine::new(ctx.clone(), None);
            return engine.analyze(layout);
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        let engine = WasmSimdScoringEngine::new(ctx.clone(), None);
        return engine.analyze(layout);
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("sve") {
            let engine = ArmSveScoringEngine::new(ctx.clone(), None);
            return engine.analyze(layout);
        }
        let engine = ArmNeonScoringEngine::new(ctx.clone(), None);
        return engine.analyze(layout);
    }

    #[cfg(not(any(target_arch = "wasm32", target_arch = "aarch64")))]
    {
        let validated = ValidatedLayout::new(&layout.keys, ctx.key_count)?;
        Ok(kernel::compute::analyze_layout(ctx, &validated))
    }
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
