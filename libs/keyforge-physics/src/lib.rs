mod analysis;
pub mod kernel;
pub mod verify; // NEW: The Sidecar

pub use analysis::fingerprint::LayoutIdentity;
pub use keyforge_model::SwapSuggestion;

use analysis::fingerprint::Fingerprinter;
use analysis::heuristics::suggest_swaps;
use kernel::compiler::Compiler;
use kernel::compute::{analyze_layout, score_layout};
pub use kernel::EngineContext;
use keyforge_model::{
    AnalysisReport, Corpus, Keyboard, Layout, OptimizationResult, Rubric, SearchConfig,
};
use keyforge_protocol::constants::SCORE_SCALE;
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
    ) -> Self {
        let ctx = Compiler::compile(keyboard, corpus, rubric, cost_overrides);
        Self { ctx }
    }

    #[instrument(skip(self, layout))]
    pub fn score(&self, layout: &Layout) -> f32 {
        let mut pos_map = vec![65535u16; 65536];
        score_layout(&self.ctx, &layout.keys, &mut pos_map) as f32 / SCORE_SCALE
    }

    #[instrument(skip(self, layout))]
    pub fn analyze(&self, layout: &Layout) -> AnalysisReport {
        analyze_layout(&self.ctx, &layout.keys)
    }

    #[instrument(skip(self, layout))]
    pub fn suggest_improvements(&self, layout: &Layout) -> Vec<SwapSuggestion> {
        suggest_swaps(&self.ctx, layout)
    }

    pub fn calculate_swap_delta(
        &self,
        layout: &[u16],
        pos_map: &[u16],
        idx_a: usize,
        idx_b: usize,
    ) -> i64 {
        kernel::compute::calculate_swap_delta(&self.ctx, layout, pos_map, idx_a, idx_b)
    }

    pub fn score_raw(&self, layout: &[u16]) -> i64 {
        let mut pos_map = vec![65535u16; 65536];
        score_layout(&self.ctx, layout, &mut pos_map)
    }

    pub fn key_count(&self) -> usize {
        self.ctx.key_count
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
    pub pinned_keys: Vec<Option<u16>>,
    pub cost_overrides: Vec<(usize, usize, f32)>,
}

#[instrument(skip(req))]
pub fn score(req: &EngineRequest) -> OptimizationResult {
    let engine = ScoringEngine::new(&req.keyboard, &req.corpus, &req.rubric, &req.cost_overrides);

    let layout = req
        .initial_layout
        .clone()
        .unwrap_or_else(|| Layout::new(vec![0; engine.context().key_count]));

    OptimizationResult {
        score: engine.score(&layout),
        layout,
    }
}

#[instrument(skip(req))]
pub fn analyze(req: &EngineRequest) -> AnalysisReport {
    let engine = ScoringEngine::new(&req.keyboard, &req.corpus, &req.rubric, &req.cost_overrides);
    let layout = req
        .initial_layout
        .clone()
        .unwrap_or_else(|| Layout::new(vec![0; engine.context().key_count]));
    engine.analyze(&layout)
}

#[instrument]
pub fn identify(layout: &Layout) -> Option<LayoutIdentity> {
    let fp = Fingerprinter;
    fp.identify(layout)
}

#[instrument(skip(req))]
pub fn suggest_improvements(req: &EngineRequest) -> Vec<SwapSuggestion> {
    let engine = ScoringEngine::new(&req.keyboard, &req.corpus, &req.rubric, &req.cost_overrides);
    let layout = req
        .initial_layout
        .clone()
        .unwrap_or_else(|| Layout::new(vec![0; engine.context().key_count]));
    engine.suggest_improvements(&layout)
}
