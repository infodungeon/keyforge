use super::{EngineCapabilities, EngineFeatures, ScoringEngine};
use crate::kernel::compute::analyze_layout;
use crate::kernel::types::ValidatedLayout;
use crate::kernel::EngineContext;
use crate::verify::DeterministicScorer;
use crate::PhysicsError;
use keyforge_model::{AnalysisReport, Corpus, Keyboard, Layout, Rubric, Score, SwapSuggestion};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub(crate) struct ExactScoringEngine {
    scorer: DeterministicScorer,
    keyboard: Arc<Keyboard>,
    corpus: Arc<Corpus>,
    ctx: EngineContext,
}

impl ExactScoringEngine {
    #[must_use]
    pub(crate) fn new(
        keyboard: Arc<Keyboard>,
        corpus: Arc<Corpus>,
        rubric: &Rubric,
        cost_model: &keyforge_model::CostModel,
        ctx: EngineContext,
    ) -> Self {
        let scorer = DeterministicScorer::new(&keyboard, rubric, cost_model);
        Self {
            scorer,
            keyboard,
            corpus,
            ctx,
        }
    }
}

impl ScoringEngine for ExactScoringEngine {
    fn name(&self) -> &'static str {
        "Exact (Oracle)"
    }

    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            is_exact: true,
            features: EngineFeatures::NONE,
        }
    }

    fn key_count(&self) -> usize {
        self.keyboard.keys.len()
    }

    fn score(&self, layout: &Layout) -> Result<Score, PhysicsError> {
        self.scorer
            .score(&self.keyboard, &self.corpus, layout.keys.as_slice())
            .map(Score)
    }

    fn score_with_scratch(
        &self,
        layout: &Layout,
        _scratch: &mut crate::kernel::compute::PhysicsScratch,
    ) -> Result<Score, PhysicsError> {
        self.score(layout)
    }

    fn score_detailed(&self, layout: &Layout) -> Result<(i64, i64, i64), PhysicsError> {
        self.scorer
            .score_detailed(&self.keyboard, &self.corpus, layout.keys.as_slice())
    }

    fn calculate_swap_delta(
        &self,
        layout: &Layout,
        pos_map: &[keyforge_model::types::KeyIndex],
        idx_a: usize,
        idx_b: usize,
    ) -> Result<i64, PhysicsError> {
        let validated = ValidatedLayout::new(&layout.keys, self.ctx.key_count)?;
        let pm = crate::kernel::compute::state::PosMap::from_slice(pos_map, self.ctx.key_count);

        crate::kernel::compute::delta::calculate_swap_delta(
            &self.ctx, &validated, &pm, idx_a, idx_b,
        )
    }

    fn analyze(&self, layout: &Layout) -> Result<AnalysisReport, PhysicsError> {
        let validated = ValidatedLayout::new(&layout.keys, self.ctx.key_count)?;
        analyze_layout(&self.ctx, &validated)
    }

    fn suggest_improvements(&self, layout: &Layout, include_thumbs: bool) -> Vec<SwapSuggestion> {
        crate::analysis::heuristics::suggest_swaps(&self.ctx, layout, include_thumbs)
    }

    fn context(&self) -> &EngineContext {
        &self.ctx
    }
}
