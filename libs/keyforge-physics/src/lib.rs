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

#[cfg(test)]
mod tests {
    use super::*;
    use keyforge_model::{Corpus, KeyNode, Keyboard, Layout, Rubric};
    use keyforge_model::types::{HandIndex, FingerIndex, RowIndex, ColIndex};
    use std::sync::Arc;

    fn setup_physics_engine() -> ScoringEngine {
        let keys = vec![
            KeyNode { index: 0, label: "A".to_string(), hand: HandIndex(0), finger: FingerIndex(1), row: RowIndex(1), col: ColIndex(0), x: 0.0, y: 1.0, is_home: true, ..Default::default() },
            KeyNode { index: 1, label: "B".to_string(), hand: HandIndex(0), finger: FingerIndex(2), row: RowIndex(1), col: ColIndex(1), x: 1.0, y: 1.0, is_home: true, ..Default::default() },
            KeyNode { index: 2, label: "C".to_string(), hand: HandIndex(0), finger: FingerIndex(3), row: RowIndex(1), col: ColIndex(2), x: 2.0, y: 1.0, is_home: true, ..Default::default() },
            KeyNode { index: 3, label: "D".to_string(), hand: HandIndex(1), finger: FingerIndex(1), row: RowIndex(1), col: ColIndex(6), x: 6.0, y: 1.0, is_home: true, ..Default::default() },
            KeyNode { index: 4, label: "E".to_string(), hand: HandIndex(0), finger: FingerIndex(1), row: RowIndex(3), col: ColIndex(0), x: 0.0, y: 3.0, is_home: false, ..Default::default() },
        ];
        let keyboard = Arc::new(Keyboard::new(keys, 1).unwrap());

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
                index: i,
                label: format!("k{}", i),
                hand: HandIndex(0),
                finger: FingerIndex(i as u8),
                row: RowIndex(0),
                col: ColIndex(i as i8),
                x: i as f32,
                y: 0.0,
                is_home: true,
                ..Default::default()
            })
            .collect();
        Keyboard::new(keys, 0).unwrap()
    }

    #[test]
    fn test_analyze_layout_comprehensive() {
        let engine = setup_physics_engine();
        let layout = Layout::new_unchecked(vec![KeyCode(0), KeyCode(1), KeyCode(2), KeyCode(3), KeyCode(4)]);
        let report = engine.analyze(&layout).unwrap();

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
            engine.analyze(&layout).unwrap();
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_swap_delta_bounds_strict() {
        let kb = setup_kb(5);
        let layout_vec: Vec<KeyCode> = (0..10u16).map(KeyCode).collect();
        let layout = Layout::new_unchecked(layout_vec);
        let corpus = Corpus::default();
        let rubric = Rubric::default();
        let engine = ScoringEngine::new(&kb, &corpus, &rubric, &[]).unwrap();

        let mut pos_map = vec![65535u16; 65536];
        for i in 0..5 {
            pos_map[i] = i as u16;
        }

        let delta = engine.calculate_swap_delta(&layout.keys, &pos_map, 0, 8).unwrap();
        assert_eq!(delta, 0);
        let delta = engine.calculate_swap_delta(&layout.keys, &pos_map, 8, 0).unwrap();
        assert_eq!(delta, 0);
    }

    #[test]
    fn test_swap_delta_reflexive_skips() {
        let kb = setup_kb(5);
        let layout = Layout::new_unchecked(vec![KeyCode(0), KeyCode(1), KeyCode(2), KeyCode(3), KeyCode(4)]);
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
        let _ = engine.calculate_swap_delta(&layout.keys, &pos_map, 0, 1).unwrap();
    }

    #[test]
    fn test_swap_delta_math_coverage() {
        let kb = setup_kb(5);
        let layout = Layout::new_unchecked(vec![KeyCode(0), KeyCode(1), KeyCode(2), KeyCode(3), KeyCode(4)]);
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
        let _ = engine.calculate_swap_delta(&layout.keys, &pos_map, 0, 3).unwrap();
    }
}
