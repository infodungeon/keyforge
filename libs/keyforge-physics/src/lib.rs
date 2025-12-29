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
use kernel::types::ValidatedLayout;
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
    ) -> Result<Self, PhysicsError> {
        let ctx = Compiler::compile(keyboard, corpus, rubric, cost_overrides)?;
        Ok(Self { ctx })
    }

    #[instrument(skip(self, layout))]
    pub fn score(&self, layout: &Layout) -> f32 {
        let validated = ValidatedLayout::new(&layout.keys, self.ctx.key_count)
            .expect("Physics Violation: Layout size mismatch");
        
        let mut pos_map = vec![65535u16; 65536];
        score_layout(&self.ctx, &validated, &mut pos_map) as f32 / SCORE_SCALE
    }

    #[instrument(skip(self, layout))]
    pub fn analyze(&self, layout: &Layout) -> AnalysisReport {
        let validated = ValidatedLayout::new(&layout.keys, self.ctx.key_count)
            .expect("Physics Violation: Layout size mismatch");
        analyze_layout(&self.ctx, &validated)
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
        let validated = ValidatedLayout::new(layout, self.ctx.key_count)
            .expect("Physics Violation: Layout size mismatch");
        kernel::compute::calculate_swap_delta(&self.ctx, &validated, pos_map, idx_a, idx_b)
    }

    pub fn score_raw(&self, layout: &[u16]) -> i64 {
        let validated = ValidatedLayout::new(layout, self.ctx.key_count)
            .expect("Physics Violation: Layout size mismatch");
        let mut pos_map = vec![65535u16; 65536];
        score_layout(&self.ctx, &validated, &mut pos_map)
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
    let engine = ScoringEngine::new(&req.keyboard, &req.corpus, &req.rubric, &req.cost_overrides)
        .expect("Physics Violation: Invalid Keyboard Geometry in Request");

    let layout = req
        .initial_layout
        .clone()
        .unwrap_or_else(|| Layout::new_unchecked(vec![0; engine.context().key_count]));

    OptimizationResult {
        score: engine.score(&layout),
        layout,
    }
}

#[instrument(skip(req))]
pub fn analyze(req: &EngineRequest) -> AnalysisReport {
    let engine = ScoringEngine::new(&req.keyboard, &req.corpus, &req.rubric, &req.cost_overrides)
        .expect("Physics Violation: Invalid Keyboard Geometry in Request");
    let layout = req
        .initial_layout
        .clone()
        .unwrap_or_else(|| Layout::new_unchecked(vec![0; engine.context().key_count]));
    engine.analyze(&layout)
}

#[instrument]
pub fn identify(layout: &Layout) -> Option<LayoutIdentity> {
    let fp = Fingerprinter;
    fp.identify(layout)
}

#[instrument(skip(req))]
pub fn suggest_improvements(req: &EngineRequest) -> Vec<SwapSuggestion> {
    let engine = ScoringEngine::new(&req.keyboard, &req.corpus, &req.rubric, &req.cost_overrides)
        .expect("Physics Violation: Invalid Keyboard Geometry in Request");
    let layout = req
        .initial_layout
        .clone()
        .unwrap_or_else(|| Layout::new_unchecked(vec![0; engine.context().key_count]));
    engine.suggest_improvements(&layout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyforge_model::{Corpus, KeyNode, Keyboard, Layout, Rubric};
    use std::sync::Arc;

    fn setup_physics_engine() -> ScoringEngine {
        let keys = vec![
            KeyNode { id: 0, label: "A".to_string(), hand: 0, finger: 1, row: 1, col: 0, x: 0.0, y: 1.0, is_home: true },
            KeyNode { id: 1, label: "B".to_string(), hand: 0, finger: 2, row: 1, col: 1, x: 1.0, y: 1.0, is_home: true },
            KeyNode { id: 2, label: "C".to_string(), hand: 0, finger: 3, row: 1, col: 2, x: 2.0, y: 1.0, is_home: true },
            KeyNode { id: 3, label: "D".to_string(), hand: 1, finger: 1, row: 1, col: 6, x: 6.0, y: 1.0, is_home: true },
            KeyNode { id: 4, label: "E".to_string(), hand: 0, finger: 1, row: 3, col: 0, x: 0.0, y: 3.0, is_home: false },
        ];
        let keyboard = Arc::new(Keyboard::new(keys, 1));

        let mut corpus = Corpus::default();
        corpus.char_freqs[0] = 100;
        corpus.char_freqs[1] = 50;
        corpus.bigrams.push((0, 4, 10));
        corpus.bigrams.push((1, 4, 5));
        corpus.trigrams.push((2, 1, 0, 20));
        corpus.trigrams.push((0, 2, 1, 15));

        let rubric = Arc::new(Rubric::default());
        ScoringEngine::new(&keyboard, &Arc::new(corpus), &rubric, &[]).unwrap()
    }

    fn setup_kb(size: usize) -> Keyboard {
        let keys: Vec<KeyNode> = (0..size)
            .map(|i| KeyNode {
                id: i,
                label: format!("k{}", i),
                hand: 0,
                finger: i as u8,
                row: 0,
                col: i as i8,
                x: i as f32,
                y: 0.0,
                is_home: true,
            })
            .collect();
        Keyboard::new(keys, 0)
    }

    #[test]
    fn test_analyze_layout_comprehensive() {
        let engine = setup_physics_engine();
        let layout = Layout::new_unchecked(vec![0, 1, 2, 3, 4]);
        let report = engine.analyze(&layout);

        assert_eq!(report.heatmap[0], 100.0);
        assert_eq!(report.heatmap[1], 50.0);
        assert!(report.sfb_ratio > 0.6 && report.sfb_ratio < 0.7);
        assert!(report.scissors > 0.0);
        assert!(report.rolls > 0.0);
        assert!(report.redirects > 0.0);
        assert_eq!(report.hand_balance, -1.0);
    }

    #[test]
    fn test_analyze_layout_empty() {
        let engine = setup_physics_engine();
        // Layout size 0 vs key count 5 -> Should panic due to ValidatedLayout
        let layout = Layout::new_unchecked(vec![]);
        let result = std::panic::catch_unwind(|| {
            engine.analyze(&layout);
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_swap_delta_bounds_strict() {
        let kb = setup_kb(5);
        let layout_vec: Vec<u16> = (0..10).collect();
        let layout = Layout::new_unchecked(layout_vec);
        let corpus = Corpus::default();
        let rubric = Rubric::default();
        let engine = ScoringEngine::new(&kb, &corpus, &rubric, &[]).unwrap();

        let mut pos_map = vec![65535u16; 65536];
        for i in 0..5 {
            pos_map[i] = i as u16;
        }

        let delta = engine.calculate_swap_delta(&layout.keys, &pos_map, 0, 8);
        assert_eq!(delta, 0);
        let delta = engine.calculate_swap_delta(&layout.keys, &pos_map, 8, 0);
        assert_eq!(delta, 0);
    }

    #[test]
    fn test_swap_delta_reflexive_skips() {
        let kb = setup_kb(5);
        let layout = Layout::new_unchecked(vec![0, 1, 2, 3, 4]);
        let mut corpus = Corpus::default();
        corpus.bigrams.push((0, 0, 100));
        corpus.trigrams.push((0, 0, 0, 100));
        corpus.trigrams.push((0, 1, 0, 100));
        let rubric = Rubric::default();
        let engine = ScoringEngine::new(&kb, &corpus, &rubric, &[]).unwrap();

        let mut pos_map = vec![65535u16; 65536];
        for i in 0..5 {
            pos_map[i as usize] = i as u16;
        }
        let _ = engine.calculate_swap_delta(&layout.keys, &pos_map, 0, 1);
    }

    #[test]
    fn test_swap_delta_math_coverage() {
        let kb = setup_kb(5);
        let layout = Layout::new_unchecked(vec![0, 1, 2, 3, 4]);
        let mut corpus = Corpus::default();
        corpus.bigrams.push((0, 1, 100));
        corpus.bigrams.push((1, 0, 100));
        corpus.trigrams.push((0, 1, 2, 100));
        corpus.trigrams.push((1, 0, 2, 100));
        corpus.trigrams.push((1, 2, 0, 100));
        let rubric = Rubric::default();
        let engine = ScoringEngine::new(&kb, &corpus, &rubric, &[]).unwrap();

        let mut pos_map = vec![65535u16; 65536];
        for i in 0..5 {
            pos_map[i as usize] = i as u16;
        }
        let _ = engine.calculate_swap_delta(&layout.keys, &pos_map, 0, 3);
    }
}
