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
mod kernel;
/// Layout verification and validity scoring.
pub mod verify; 
/// Physics-specific error types.
pub mod errors;

pub use analysis::fingerprint::LayoutIdentity;
pub use keyforge_model::SwapSuggestion;

use analysis::fingerprint::Fingerprinter;
use analysis::heuristics::suggest_swaps;
use kernel::compiler::Compiler;
pub use errors::PhysicsError;
use kernel::compute::{analyze_layout, score_layout};
pub use kernel::EngineContext;
use kernel::types::{KeyCode, ValidatedLayout};
use keyforge_model::{
    AnalysisReport, Corpus, Keyboard, Layout, OptimizationResult, Rubric, SearchConfig, CostModel
};
use keyforge_model::constants::SCORE_SCALE;
use std::sync::Arc;
use tracing::instrument;

/// The core evaluation engine for `KeyForge` layouts.
///
/// `ScoringEngine` encapsulates the compiled physics kernel, including 
/// pre-calculated travel costs and frequency-weighted optimization targets.
#[derive(Debug)]
pub struct ScoringEngine {
    ctx: EngineContext,
}

impl ScoringEngine {
    /// Compiles a new scoring engine from the provided keyboard, corpus, and rubric.
    ///
    /// This performs expensive pre-computations and returns an engine ready 
    /// for high-performance evaluations.
    ///
    /// # Errors
    ///
    /// Returns a `PhysicsError::Config` if the compilation of the physics kernel fails.
    pub fn new(
        keyboard: &Keyboard,
        corpus: &Corpus,
        rubric: &Rubric,
        cost_model: &CostModel,
    ) -> Result<Self, PhysicsError> {
        let ctx = Compiler::compile(keyboard, corpus, rubric, cost_model)?;
        Ok(Self { ctx })
    }

    /// Evaluates the physical cost score for a given layout.
    ///
    /// Lower scores indicate better ergonomic performance. The result is 
    /// normalized for comparison across different corpora and rubrics.
    ///
    /// # Errors
    ///
    /// Returns a `PhysicsError` if the scoring calculation fails (e.g., invalid layout length).
    #[allow(clippy::cast_precision_loss)]
    pub fn score(&self, layout: &Layout) -> Result<f32, PhysicsError> {
        let raw_score_scaled = self.score_raw(&layout.keys)?;
        let raw_score = raw_score_scaled as f32 / SCORE_SCALE;
        
        let total_freq: u64 = self.ctx.char_freqs.iter().sum();
        let norm_factor = if total_freq > 0 {
            100_000.0 / total_freq as f32
        } else {
            1.0
        };
        
        Ok(raw_score * norm_factor)
    }

    /// Analyzes a layout and returns a detailed report of its performance.
    ///
    /// # Errors
    ///
    /// Returns a `PhysicsError` if the analysis fails.
    #[instrument(skip(self, layout))]
    pub fn analyze(&self, layout: &Layout) -> Result<AnalysisReport, PhysicsError> {
        let validated = ValidatedLayout::new(&layout.keys, self.ctx.key_count)?;
        Ok(analyze_layout(&self.ctx, &validated))
    }

    #[instrument(skip(self, layout))]
    #[must_use] 
    pub fn suggest_improvements(&self, layout: &Layout, include_thumbs: bool) -> Vec<SwapSuggestion> {
        suggest_swaps(&self.ctx, layout, include_thumbs)
    }

    /// Calculates the change in score resulting from swapping two keys.
    ///
    /// This is an optimized operation used during local search.
    ///
    /// # Errors
    ///
    /// Returns a `PhysicsError::Config` if the layout is invalid for the current engine context.
    pub fn calculate_swap_delta(
        &self,
        layout: &[KeyCode],
        _pos_map: &[u16],
        idx_a: usize,
        idx_b: usize,
    ) -> Result<i64, PhysicsError> {
        let validated = ValidatedLayout::new(layout, self.ctx.key_count)?;
        let mut scratch = Box::new(kernel::compute::PhysicsScratch::new());
        
        let pm = kernel::compute::PosMap::from_scratch(
            layout,
            self.ctx.key_count,
            scratch.starts.as_mut_slice(),
            scratch.counts.as_mut_slice(),
            scratch.indices.as_mut_slice(),
            scratch.current_offsets.as_mut_slice(),
            &mut scratch.used_keys,
        );
        
        Ok(kernel::compute::calculate_swap_delta(&self.ctx, &validated, &pm, idx_a, idx_b))
    }

    /// Returns the raw, unweighted physics score for a layout.
    ///
    /// # Errors
    ///
    /// Returns a `PhysicsError` if the scoring calculation fails.
    #[allow(clippy::cast_precision_loss)]
    pub fn score_raw(&self, layout: &[KeyCode]) -> Result<i64, PhysicsError> {
        let validated = ValidatedLayout::new(layout, self.ctx.key_count)?;
        let mut scratch = Box::new(kernel::compute::PhysicsScratch::new());
        Ok(score_layout(&self.ctx, &validated, &mut scratch))
    }

    /// Returns the total number of keys supported by this engine.
    #[must_use] 
    pub fn key_count(&self) -> usize {
        self.ctx.key_count
    }

    /// Returns the total number of trigrams used for scoring.
    #[must_use] 
    pub fn trigram_count(&self) -> usize {
        self.ctx.trigram_freqs.len()
    }

    /// Returns a reference to the internal engine context.
    #[must_use] 
    pub fn context(&self) -> &EngineContext {
        &self.ctx
    }
}

/// A request structure for performing common engine operations.
///
/// This structure bundles all necessary data to instantiate a `ScoringEngine` 
/// and perform a task like scoring or analysis.
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
pub fn score(req: &EngineRequest) -> Result<OptimizationResult, PhysicsError> {
    let engine = ScoringEngine::new(&req.keyboard, &req.corpus, &req.rubric, &req.cost_model)?;
    let layout = req
        .initial_layout
        .clone()
        .unwrap_or_else(|| Layout::new_unchecked(vec![KeyCode(0); engine.context().key_count]));
    Ok(OptimizationResult {
        score: engine.score(&layout)?,
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
    let engine = ScoringEngine::new(&req.keyboard, &req.corpus, &req.rubric, &req.cost_model)?;
    let layout = req
        .initial_layout
        .clone()
        .unwrap_or_else(|| Layout::new_unchecked(vec![KeyCode(0); engine.context().key_count]));
    engine.analyze(&layout)
}

/// Identifies a layout by comparing it to known standards.
#[instrument]
pub fn identify(layout: &Layout) -> Option<LayoutIdentity> {
    Fingerprinter::identify(layout)
}

/// Suggests improvements for the layout described in the request.
///
/// # Errors
///
/// Returns a `PhysicsError` if the engine initialization fails.
#[instrument(skip(req))]
pub fn suggest_improvements(req: &EngineRequest) -> Result<Vec<SwapSuggestion>, PhysicsError> {
    let engine = ScoringEngine::new(&req.keyboard, &req.corpus, &req.rubric, &req.cost_model)?;
    let layout = req
        .initial_layout
        .clone()
        .unwrap_or_else(|| Layout::new_unchecked(vec![KeyCode(0); engine.context().key_count]));
    Ok(engine.suggest_improvements(&layout, req.config.include_thumbs()))
}
