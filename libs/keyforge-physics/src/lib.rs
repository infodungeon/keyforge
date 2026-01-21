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
/// Physics-specific error types.
pub mod error;
mod kernel;
/// Layout verification and validity scoring.
pub mod verify;
pub mod engines;

pub use analysis::fingerprint::LayoutIdentity;
pub use keyforge_model::SwapSuggestion;

use analysis::fingerprint::Fingerprinter;
use analysis::heuristics::suggest_swaps;
pub use engines::{EngineCapabilities, ScoringEngine};
pub use engines::intel_comet_lake::IntelEngineConfig;
use engines::{exact::ExactScoringEngine, generic::GenericScoringEngine, intel_comet_lake::IntelScoringEngine};
pub use error::PhysicsError;
use kernel::compiler::Compiler;
use kernel::compute::analyze_layout;
use kernel::types::ValidatedLayout;
pub use kernel::EngineContext;
use keyforge_model::{
    AnalysisReport, Corpus, CostModel, KeyCode, Keyboard, Layout, Rubric, ScoringResult, SearchConfig,
};
use std::sync::Arc;
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
            rubric.clone(),
            cost_model,
            ctx,
        )))
    }

    /// Compiles a new **Intel Comet Lake** scoring engine.
    ///
    /// This engine uses AVX2 optimizations and cache-aware access patterns.
    /// It is only safe to use on compatible hardware (checked by caller).
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
}

/// A request structure for performing common engine operations.
#[derive(Clone, Debug)]
pub struct EngineRequest {
    /// The physical keyboard geometry.
    pub keyboard: Arc<Keyboard>,
    /// The language statistics to use.
    pub corpus: Arc<Corpus>,
    /// The ergonomic weights to apply.
    pub rubric: Arc<Rubric>,
    /// The cost model to use.
    pub cost_model: Arc<CostModel>,
    /// Optimization and search parameters.
    pub config: SearchConfig,
    /// The starting layout for the operation.
    pub initial_layout: Option<Layout>,
    /// Keys that must remain in their initial positions.
    pub pinned_keys: Vec<Option<KeyCode>>,
}

/// Performs a one-off scoring operation for the given request.
///
/// # Errors
///
/// Returns a `PhysicsError` if the engine initialization or scoring fails.
#[instrument(skip(req))]
pub fn score(req: &EngineRequest) -> Result<ScoringResult, PhysicsError> {
    let engine = EngineFactory::new_generic(&req.keyboard, &req.corpus, &req.rubric, &req.cost_model)?;
    let layout = req
        .initial_layout
        .clone()
        .unwrap_or_else(|| Layout::new_unchecked(vec![KeyCode::EMPTY; engine.key_count()]));
        
    Ok(ScoringResult {
        score: engine.score(&layout)?.to_f32(),
        layout,
    })
}

/// Performs a one-off analysis operation for the given request.
///
/// # Errors
///
/// Returns a `PhysicsError` if the engine initialization or analysis fails.
#[instrument(skip(req))]
pub fn analyze(req: &EngineRequest) -> Result<AnalysisReport, PhysicsError> {
    let ctx = Compiler::compile(&req.keyboard, &req.corpus, &req.rubric, &req.cost_model)?;
    let layout = req
        .initial_layout
        .clone()
        .unwrap_or_else(|| Layout::new_unchecked(vec![KeyCode::EMPTY; ctx.key_count]));
    analyze_with_context(&ctx, &layout)
}

/// Identifies a layout by comparing it to known standards.
#[instrument]
pub fn identify(layout: &Layout) -> Option<LayoutIdentity> {
    Fingerprinter::identify(layout)
}

/// Suggests improvements for the layout described in the request.
#[instrument(skip(req))]
pub fn suggest_improvements(req: &EngineRequest) -> Result<Vec<SwapSuggestion>, PhysicsError> {
    let ctx = Compiler::compile(&req.keyboard, &req.corpus, &req.rubric, &req.cost_model)?;
    let layout = req
        .initial_layout
        .clone()
        .unwrap_or_else(|| Layout::new_unchecked(vec![KeyCode::EMPTY; ctx.key_count]));
    
    let raw_suggestions = suggest_improvements_with_context(&ctx, &layout, req.config.include_thumbs());
    
    // Filter out suggestions that involve pinned keys
    let filtered: Vec<SwapSuggestion> = raw_suggestions
        .into_iter()
        .filter(|s| {
            let pin_a = req.pinned_keys.get(s.index_a).and_then(|p| p.as_ref()).is_some();
            let pin_b = req.pinned_keys.get(s.index_b).and_then(|p| p.as_ref()).is_some();
            !pin_a && !pin_b
        })
        .collect();

    Ok(filtered)
}

/// Analyzes a layout and returns a detailed report.
pub fn analyze_with_context(ctx: &EngineContext, layout: &Layout) -> Result<AnalysisReport, PhysicsError> {
    let validated = ValidatedLayout::new(&layout.keys, ctx.key_count)?;
    Ok(analyze_layout(ctx, &validated))
}

/// Suggests improvements for the layout.
pub fn suggest_improvements_with_context(
    ctx: &EngineContext,
    layout: &Layout,
    include_thumbs: bool,
) -> Vec<SwapSuggestion> {
    suggest_swaps(ctx, layout, include_thumbs)
}