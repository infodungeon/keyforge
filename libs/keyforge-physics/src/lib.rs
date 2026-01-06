// Copyright (c) 2025 KeyForge Contributors
//
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
mod analysis;
mod kernel;
pub mod verify; 
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
    AnalysisReport, Corpus, Keyboard, Layout, OptimizationResult, Rubric, SearchConfig,
};
use keyforge_model::constants::SCORE_SCALE;
use std::sync::Arc;
use tracing::instrument;

pub struct ScoringEngine {
    ctx: EngineContext,
}

impl ScoringEngine {
    pub fn new(
        keyboard: &Keyboard,
        corpus: &Corpus,
        rubric: &Rubric,
        cost_overrides: &[(usize, usize, f32)],
    ) -> Result<Self, PhysicsError> {
        let ctx = Compiler::compile(keyboard, corpus, rubric, cost_overrides)?;
        Ok(Self { ctx })
    }

    #[instrument(skip(self, layout))]
    pub fn score(&self, layout: &Layout) -> Result<f32, PhysicsError> {
        let validated = ValidatedLayout::new(&layout.keys, self.ctx.key_count)?;
        let mut pos_map = vec![65535u16; 65536];
        Ok(score_layout(&self.ctx, &validated, &mut pos_map) as f32 / SCORE_SCALE)
    }

    #[instrument(skip(self, layout))]
    pub fn analyze(&self, layout: &Layout) -> Result<AnalysisReport, PhysicsError> {
        let validated = ValidatedLayout::new(&layout.keys, self.ctx.key_count)?;
        Ok(analyze_layout(&self.ctx, &validated))
    }

    #[instrument(skip(self, layout))]
    pub fn suggest_improvements(&self, layout: &Layout) -> Vec<SwapSuggestion> {
        suggest_swaps(&self.ctx, layout)
    }

    pub fn calculate_swap_delta(
        &self,
        layout: &[KeyCode],
        pos_map: &[u16],
        idx_a: usize,
        idx_b: usize,
    ) -> Result<i64, PhysicsError> {
        let validated = ValidatedLayout::new(layout, self.ctx.key_count)?;
        Ok(kernel::compute::calculate_swap_delta(&self.ctx, &validated, pos_map, idx_a, idx_b))
    }

    pub fn score_raw(&self, layout: &[KeyCode]) -> Result<i64, PhysicsError> {
        let validated = ValidatedLayout::new(layout, self.ctx.key_count)?;
        let mut pos_map = vec![65535u16; 65536];
        Ok(score_layout(&self.ctx, &validated, &mut pos_map))
    }

    pub fn key_count(&self) -> usize {
        self.ctx.key_count
    }

    pub fn trigram_count(&self) -> usize {
        self.ctx.trigram_freqs.len()
    }

    pub fn context(&self) -> &EngineContext {
        &self.ctx
    }
}

#[derive(Clone)]
pub struct EngineRequest {
    pub keyboard: Arc<Keyboard>,
    pub corpus: Arc<Corpus>,
    pub rubric: Arc<Rubric>,
    pub config: SearchConfig,
    pub initial_layout: Option<Layout>,
    pub pinned_keys: Vec<Option<KeyCode>>,
    pub cost_overrides: Vec<(usize, usize, f32)>,
}

#[instrument(skip(req))]
pub fn score(req: &EngineRequest) -> Result<OptimizationResult, PhysicsError> {
    let engine = ScoringEngine::new(&req.keyboard, &req.corpus, &req.rubric, &req.cost_overrides)?;
    let layout = req
        .initial_layout
        .clone()
        .unwrap_or_else(|| Layout::new_unchecked(vec![KeyCode(0); engine.context().key_count]));
    Ok(OptimizationResult {
        score: engine.score(&layout)?,
        layout,
    })
}

#[instrument(skip(req))]
pub fn analyze(req: &EngineRequest) -> Result<AnalysisReport, PhysicsError> {
    let engine = ScoringEngine::new(&req.keyboard, &req.corpus, &req.rubric, &req.cost_overrides)?;
    let layout = req
        .initial_layout
        .clone()
        .unwrap_or_else(|| Layout::new_unchecked(vec![KeyCode(0); engine.context().key_count]));
    engine.analyze(&layout)
}

#[instrument]
pub fn identify(layout: &Layout) -> Option<LayoutIdentity> {
    let fp = Fingerprinter;
    fp.identify(layout)
}

#[instrument(skip(req))]
pub fn suggest_improvements(req: &EngineRequest) -> Result<Vec<SwapSuggestion>, PhysicsError> {
    let engine = ScoringEngine::new(&req.keyboard, &req.corpus, &req.rubric, &req.cost_overrides)?;
    let layout = req
        .initial_layout
        .clone()
        .unwrap_or_else(|| Layout::new_unchecked(vec![KeyCode(0); engine.context().key_count]));
    Ok(engine.suggest_improvements(&layout))
}