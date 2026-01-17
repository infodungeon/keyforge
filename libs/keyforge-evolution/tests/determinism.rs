// libs/keyforge-evolution/tests/determinism.rs

use keyforge_evolution::{optimize_with_callback, ProgressCallback};
use keyforge_model::{Corpus, KeyNode, Keyboard, Layout, Rubric, SearchConfig, KeyCode, CostModel};
use keyforge_model::types::{HandIndex, FingerIndex, RowIndex, ColIndex};
use keyforge_physics::EngineRequest;
use std::sync::Arc;

fn mock_cost_model() -> CostModel {
    let json = r#"{
        "meta": { "version": "2.0", "description": "Test", "unit": "pts" },
        "models": {
            "model_a_row_staggered": {
                "description": "Test Model",
                "static_costs": {
                    "universal_hand": {
                        "thumb": { "pos_1": 100.0 },
                        "index": { "base": { "r0": 100.0 } },
                        "middle": { "base": { "r0": 100.0 } },
                        "ring": { "base": { "r0": 100.0 } },
                        "pinky": { "base": { "r0": 100.0 } }
                    }
                }
            }
        },
        "dynamic_rules": { "sequence_modifiers": {}, "penalties": {}, "constraints": {} }
    }"#;
    serde_json::from_str(json).unwrap()
}

struct OracleCallback {
    keyboard: Arc<Keyboard>,
    corpus: Arc<Corpus>,
    rubric: Arc<Rubric>,
    cost_matrix: Vec<(usize, usize, f32)>,
}

impl ProgressCallback for OracleCallback {
    fn on_progress(&self, step: usize, score: f32, layout_keys: &[KeyCode], _ips: f32) -> bool {
        let layout = Layout::new_unchecked(layout_keys.to_vec());
        let reference_score = keyforge_physics::verify::DeterministicScorer::score_raw(
            &self.keyboard, 
            &self.corpus, 
            &self.rubric, 
            &layout, 
            &self.cost_matrix
        );
        let epsilon = 1e-4;
        if (score - reference_score).abs() > epsilon {
            panic!(
                "Shadow Execution Failure at step {}: Fast Score ({}) != Reference Score ({})",
                step, score, reference_score
            );
        }
        true
    }
}

fn create_mock_corpus() -> Corpus {
    let mut char_freqs = vec![0; 65536];
    char_freqs[0] = 100; char_freqs[1] = 100; char_freqs[2] = 50;
    Corpus {
        bigrams: vec![(0, 1, 100), (1, 2, 50), (2, 3, 25)],
        trigrams: vec![], char_freqs, words: vec![],
    }
}

fn create_mock_keyboard() -> Keyboard {
    let keys = vec![
        KeyNode { index: 0, x: 0.0, y: 0.0, row: RowIndex(0), col: ColIndex(0), hand: HandIndex(0), finger: FingerIndex(1), label: "0".to_string(), is_home: false, ..Default::default() },
        KeyNode { index: 1, x: 1.0, y: 0.0, row: RowIndex(0), col: ColIndex(1), hand: HandIndex(0), finger: FingerIndex(2), label: "1".to_string(), is_home: false, ..Default::default() },
        KeyNode { index: 2, x: 2.0, y: 0.0, row: RowIndex(0), col: ColIndex(2), hand: HandIndex(0), finger: FingerIndex(3), label: "2".to_string(), is_home: false, ..Default::default() },
        KeyNode { index: 3, x: 3.0, y: 0.0, row: RowIndex(0), col: ColIndex(3), hand: HandIndex(0), finger: FingerIndex(4), label: "3".to_string(), is_home: false, ..Default::default() },
    ];
    Keyboard::new(keys, 0).unwrap()
}

/// Intent: Verify parity between the optimized ScoringEngine and the naive DeterministicScorer during full evolution.
/// Expected: Final layout score must match the Oracle result within tolerance.
#[test]
#[ignore = "DeterministicScorer in physics crate needs to be updated to support CostModel correctly"]
fn test_oracle_pattern_match() {
    let keyboard = Arc::new(create_mock_keyboard());
    let corpus = Arc::new(create_mock_corpus());
    let rubric = Arc::new(Rubric::default());
    let cm = Arc::new(mock_cost_model());
    
    let config = SearchConfig::Annealing {
        steps: 2000, start_temp: 10.0, end_temp: 0.1, seed: 42,
        patience: 100, reheats: 0, reheat_factor: 1.0, include_thumbs: false,
    };

    let req = EngineRequest {
        keyboard: keyboard.clone(),
        corpus: corpus.clone(),
        rubric: rubric.clone(),
        cost_model: cm.clone(),
        config,
        initial_layout: None,
        pinned_keys: vec![],
    };

    let callback = OracleCallback {
        keyboard: keyboard.clone(),
        corpus: corpus.clone(),
        rubric: rubric.clone(),
        cost_matrix: vec![],
    };

    let result = optimize_with_callback(&req, callback).unwrap();

    let final_reference = keyforge_physics::verify::DeterministicScorer::score(
        &req.keyboard, &req.corpus, &req.rubric, &result.layout, &[]
    );
    
    assert!((result.score - final_reference).abs() < 1e-4);
}
