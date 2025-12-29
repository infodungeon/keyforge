use keyforge_evolution::{optimize_with_callback, ProgressCallback};
use keyforge_model::{Corpus, Keyboard, Layout, Rubric, SearchConfig, KeyCode};
use keyforge_physics::{verify::DeterministicScorer, EngineRequest};
use std::sync::Arc;

struct OracleCallback {
    keyboard: Arc<Keyboard>,
    corpus: Arc<Corpus>,
    rubric: Arc<Rubric>,
}

impl ProgressCallback for OracleCallback {
    fn on_progress(&self, step: usize, score: f32, layout_keys: &[KeyCode], _ips: f32) -> bool {
        // Run the "Slow, Simple, Correct" reference implementation
        // Note: Reconstructing Layout from Keys for the verifier
        let layout = Layout::new_unchecked(layout_keys.to_vec());
        
        // FIX: Added empty overrides argument &[]
        let reference_score = DeterministicScorer::score(
            &self.keyboard, 
            &self.corpus, 
            &self.rubric, 
            &layout, 
            &[]
        );

        // Verification: Fast != Slow => Optimization Flawed
        let epsilon = 1e-4;
        if (score - reference_score).abs() > epsilon {
            panic!(
                "Shadow Execution Failure at step {}: Fast Score ({}) != Reference Score ({})",
                step, score, reference_score
            );
        }
        
        // Continue optimization
        true
    }
}

#[test]
fn test_oracle_pattern_match() {
    let keyboard = Arc::new(create_mock_keyboard());
    let corpus = Arc::new(create_mock_corpus());
    let rubric = Arc::new(Rubric::default());
    
    let config = SearchConfig::Annealing {
        steps: 2000,
        start_temp: 10.0,
        end_temp: 0.1,
        seed: 42,
        patience: 100,
        reheats: 0,
        reheat_factor: 1.0,
    };

    let req = EngineRequest {
        keyboard: keyboard.clone(),
        corpus: corpus.clone(),
        rubric: rubric.clone(),
        config,
        initial_layout: None,
        pinned_keys: vec![],
        cost_overrides: vec![],
    };

    let callback = OracleCallback {
        keyboard: keyboard.clone(),
        corpus: corpus.clone(),
        rubric: rubric.clone(),
    };

    // Run the optimization with the oracle watching
    let result = optimize_with_callback(&req, callback);

    // Final check
    let final_reference = DeterministicScorer::score(
        &req.keyboard, 
        &req.corpus, 
        &req.rubric, 
        &result.layout, 
        &[]
    );
    
    assert!(
        (result.score - final_reference).abs() < 1e-4,
        "Final result divergence: {} vs {}",
        result.score,
        final_reference
    );
}

fn create_mock_corpus() -> Corpus {
    let mut char_freqs = vec![0; 65536];
    // Fill some freqs so it's not empty
    char_freqs[0] = 100;
    char_freqs[1] = 100;
    char_freqs[2] = 50;

    Corpus {
        bigrams: vec![
            (0, 1, 100), // A-B
            (1, 2, 50),  // B-C
            (2, 3, 25),  // C-D
        ],
        trigrams: vec![],
        char_freqs,
        words: vec![],
    }
}

use keyforge_model::types::{HandIndex, FingerIndex, RowIndex, ColIndex};

fn create_mock_keyboard() -> Keyboard {
    // Construct a simple 4-key keyboard for the corpus
    use keyforge_model::KeyNode;
    let keys = vec![
        KeyNode { index: 0, x: 0.0, y: 0.0, row: RowIndex(0), col: ColIndex(0), hand: HandIndex(0), finger: FingerIndex(1), label: "0".to_string(), is_home: false, ..Default::default() },
        KeyNode { index: 1, x: 1.0, y: 0.0, row: RowIndex(0), col: ColIndex(1), hand: HandIndex(0), finger: FingerIndex(2), label: "1".to_string(), is_home: false, ..Default::default() },
        KeyNode { index: 2, x: 2.0, y: 0.0, row: RowIndex(0), col: ColIndex(2), hand: HandIndex(0), finger: FingerIndex(3), label: "2".to_string(), is_home: false, ..Default::default() },
        KeyNode { index: 3, x: 3.0, y: 0.0, row: RowIndex(0), col: ColIndex(3), hand: HandIndex(0), finger: FingerIndex(4), label: "3".to_string(), is_home: false, ..Default::default() },
    ];
    Keyboard::new(keys, 0).unwrap()
}
