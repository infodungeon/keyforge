// libs/keyforge-physics/src/lib.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You    may obtain a copy of the License at
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
pub mod context;
pub mod engines;
pub mod error;
/// Reference ghost models for verification.
pub mod ghost;
pub mod kernel;
pub mod verify;

pub use context::{Compiled, ScoringContext, Uncompiled};

// --- RE-EXPORTS ---

// --- RE-EXPORTS ---

pub use analysis::heuristics::suggest_swaps;
pub use keyforge_model::layout::LayoutIdentity;

// Re-export unified engine configuration
pub use keyforge_model::config::EngineConfig;

pub use engines::{EngineCapabilities, EngineFeatures, ScoringEngine};
pub use error::PhysicsError;
pub use kernel::types::ValidatedLayout;
pub use kernel::EngineContext;

// Concrete implementations are hidden
use engines::arm_neon::ArmNeonScoringEngine;
use engines::arm_sve::ArmSveScoringEngine;
use engines::exact::ExactScoringEngine;
use engines::generic::GenericScoringEngine as ScalarScoringEngine;
use engines::intel_avx512::Avx512ScoringEngine;
use engines::intel_comet_lake::IntelScoringEngine;
use engines::wasm_simd::WasmSimdScoringEngine;

// Re-export analysis types from keyforge-model for convenience
pub use keyforge_model::{AnalysisReport, SwapSuggestion};

use keyforge_model::Layout;
use tracing::instrument;

/// Type alias for backward compatibility during the transition to typestate.
pub type EngineCompilationContext = ScoringContext<Uncompiled>;

/// A factory for creating high-performance scoring engines.
#[derive(Debug, Default)]
pub struct EngineFactory;

impl EngineFactory {
    /// Compiles a new **Scalar** (generic) scoring engine.
    ///
    /// # Errors
    /// Returns `PhysicsError` if compilation fails.
    #[instrument(skip_all, fields(kb = %ctx.state.keyboard.kb_type))]
    pub fn new_scalar(
        ctx: &EngineCompilationContext,
    ) -> Result<Box<dyn ScoringEngine>, PhysicsError> {
        let compiled = ctx.clone().compile()?;
        Ok(Box::new(ScalarScoringEngine::new(compiled.state.inner)))
    }

    /// Compiles a new **Exact** (Oracle) scoring engine.
    ///
    /// # Errors
    /// Returns `PhysicsError` if compilation fails.
    #[instrument(skip_all, fields(kb = %ctx.state.keyboard.kb_type))]
    pub fn new_exact(
        ctx: &EngineCompilationContext,
    ) -> Result<Box<dyn ScoringEngine>, PhysicsError> {
        let compiled = ctx.clone().compile()?;
        Ok(Box::new(ExactScoringEngine::new(
            ctx.state.keyboard.clone(),
            ctx.state.corpus.clone(),
            &ctx.state.rubric,
            &ctx.state.cost_model,
            compiled.state.inner,
        )))
    }

    /// Compiles a new generic engine (alias for scalar).
    ///
    /// # Errors
    /// Returns `PhysicsError` if compilation fails.
    #[instrument(skip_all)]
    pub fn new_generic(
        ctx: &EngineCompilationContext,
    ) -> Result<Box<dyn ScoringEngine>, PhysicsError> {
        Self::new_scalar(ctx)
    }

    /// Compiles the most optimized engine available for the current hardware.
    ///
    /// # Errors
    /// Returns `PhysicsError` if compilation fails.
    #[instrument(skip_all, fields(kb = %ctx.state.keyboard.kb_type))]
    pub fn new_optimized(
        ctx: &EngineCompilationContext,
    ) -> Result<Box<dyn ScoringEngine>, PhysicsError> {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            if is_x86_feature_detected!("avx512f")
                && is_x86_feature_detected!("avx512dq")
                && is_x86_feature_detected!("avx512bw")
            {
                return Self::new_intel_avx512(ctx, Some(ctx.state.engine_config));
            }
            if is_x86_feature_detected!("avx2") {
                return Self::new_intel_comet_lake(ctx, Some(ctx.state.engine_config));
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            if std::arch::is_aarch64_feature_detected!("sve") {
                return Self::new_arm_sve(ctx, Some(ctx.state.engine_config));
            }
            return Self::new_arm_neon(ctx, Some(ctx.state.engine_config));
        }

        #[cfg(target_arch = "wasm32")]
        {
            return Self::new_wasm_simd(ctx, Some(ctx.state.engine_config));
        }

        Self::new_scalar(ctx)
    }

    /// Compiles a new **WASM SIMD** scoring engine.
    ///
    /// # Errors
    /// Returns `PhysicsError` if compilation fails.
    #[instrument(skip_all)]
    pub fn new_wasm_simd(
        ctx: &EngineCompilationContext,
        config: Option<EngineConfig>,
    ) -> Result<Box<dyn ScoringEngine>, PhysicsError> {
        let compiled = ctx.clone().compile()?;
        Ok(Box::new(WasmSimdScoringEngine::new(
            compiled.state.inner,
            config,
        )))
    }

    /// Compiles a new **Intel AVX-512** scoring engine.
    ///
    /// # Errors
    /// Returns `PhysicsError` if compilation fails.
    #[instrument(skip_all)]
    pub fn new_intel_avx512(
        ctx: &EngineCompilationContext,
        config: Option<EngineConfig>,
    ) -> Result<Box<dyn ScoringEngine>, PhysicsError> {
        let compiled = ctx.clone().compile()?;
        Ok(Box::new(Avx512ScoringEngine::new(
            compiled.state.inner,
            config,
        )))
    }

    /// Compiles a new **Intel AVX2** scoring engine.
    ///
    /// # Errors
    /// Returns `PhysicsError` if compilation fails.
    #[instrument(skip_all)]
    pub fn new_intel_comet_lake(
        ctx: &EngineCompilationContext,
        config: Option<EngineConfig>,
    ) -> Result<Box<dyn ScoringEngine>, PhysicsError> {
        let compiled = ctx.clone().compile()?;
        Ok(Box::new(IntelScoringEngine::new(
            compiled.state.inner,
            config,
        )))
    }

    /// Compiles a new **ARM SVE** scoring engine.
    ///
    /// # Errors
    /// Returns `PhysicsError` if compilation fails.
    #[instrument(skip_all)]
    pub fn new_arm_sve(
        ctx: &EngineCompilationContext,
        config: Option<EngineConfig>,
    ) -> Result<Box<dyn ScoringEngine>, PhysicsError> {
        let compiled = ctx.clone().compile()?;
        Ok(Box::new(ArmSveScoringEngine::new(
            compiled.state.inner,
            config,
        )))
    }

    /// Compiles a new **ARM NEON** scoring engine.
    ///
    /// # Errors
    /// Returns `PhysicsError` if compilation fails.
    #[instrument(skip_all)]
    pub fn new_arm_neon(
        ctx: &EngineCompilationContext,
        config: Option<EngineConfig>,
    ) -> Result<Box<dyn ScoringEngine>, PhysicsError> {
        let compiled = ctx.clone().compile()?;
        Ok(Box::new(ArmNeonScoringEngine::new(
            compiled.state.inner,
            config,
        )))
    }
}

/// Identifies a layout by comparing it to known standards.
#[instrument(skip_all)]
pub fn identify(layout: &Layout) -> Option<LayoutIdentity> {
    layout.identify()
}

/// Analyzes a layout and returns a detailed report.
///
/// # Errors
/// Returns `PhysicsError` if the layout is invalid for the context.
#[instrument(skip_all)]
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

    let validated = ValidatedLayout::new(layout.keys(), ctx.key_count)?;
    kernel::compute::analyze_layout(ctx, &validated)
}

/// Suggests improvements for the layout.
#[instrument(skip_all)]
#[must_use]
pub fn suggest_improvements_with_context(
    ctx: &EngineContext,
    layout: &Layout,
    include_thumbs: bool,
) -> Vec<SwapSuggestion> {
    suggest_swaps(ctx, layout, include_thumbs)
}
