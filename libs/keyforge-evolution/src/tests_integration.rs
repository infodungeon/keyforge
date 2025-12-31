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
use crate::supervisor::AnnealingConfig;
use crate::supervisor::strategies::{CoolingAnnealing, GroupMutation};
use crate::supervisor::traits::{MutationAction, MutationOperator, MutationProposal};
use crate::supervisor::Optimizer;
use crate::{optimize_with_callback, ProgressCallback, EvolutionError};
use keyforge_model::{Corpus, KeyNode, Keyboard, Layout, Rubric, SearchConfig, KeyCode, KeyIndex};
use keyforge_model::types::{HandIndex, FingerIndex, RowIndex, ColIndex};
use keyforge_physics::{EngineRequest, ScoringEngine};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

// --- Helper Functions ---
fn setup_env_integrated() -> (Arc<Keyboard>, Arc<Corpus>, Arc<Rubric>) {
    let keys = vec![
        KeyNode { index: 0, label: "k0".to_string(), hand: HandIndex(0), finger: FingerIndex(1), row: RowIndex(0), col: ColIndex(0), x: 0.0, y: 0.0, is_home: true, ..Default::default() },
        KeyNode { index: 1, label: "k1".to_string(), hand: HandIndex(0), finger: FingerIndex(2), row: RowIndex(0), col: ColIndex(1), x: 1.0, y: 0.0, is_home: true, ..Default::default() },
        KeyNode { index: 2, label: "k2".to_string(), hand: HandIndex(0), finger: FingerIndex(3), row: RowIndex(0), col: ColIndex(2), x: 2.0, y: 0.0, is_home: true, ..Default::default() },
    ];
    (Arc::new(Keyboard::new(keys, 0).unwrap()), Arc::new(Corpus::default()), Arc::new(Rubric::default()))
}

fn setup_test_engine(size: usize) -> ScoringEngine {
    let keys: Vec<_> = (0..size)
        .map(|i| KeyNode {
            index: i,
            label: format!("k{}", i),
            hand: HandIndex((i % 2) as u8),
            finger: FingerIndex((i % 5) as u8),
            row: RowIndex((i / 10) as i8),
            col: ColIndex((i % 10) as i8),
            x: (i % 10) as f32,
            y: (i / 10) as f32,
            is_home: false,
            ..Default::default()
        })
        .collect();
    let kb = Keyboard::new(keys, 1).unwrap();
    let mut corpus = Corpus::default();
    for i in 0..size {
        corpus.char_freqs[i] = (i * 10) as u32;
        if i + 1 < size {
            corpus.bigrams.push((i as u16, (i + 1) as u16, 100));
        }
    }
    ScoringEngine::new(&kb, &corpus, &Rubric::default(), &[]).unwrap()
}

// --- Supervisor Logic Tests ---

#[test]
fn test_monotonicity_zero_temp() {
    let engine = setup_test_engine(30);
    let mutation = GroupMutation { unlocked_indices: (0..30).collect() };
    let acceptance = CoolingAnnealing;

    struct ScoreCheckCallback {
        last_score: std::sync::Mutex<f32>,
        failed: AtomicBool,
    }
    impl ProgressCallback for ScoreCheckCallback {
        fn on_progress(&self, _epoch: usize, score: f32, _layout: &[KeyCode], _ips: f32) -> bool {
            let mut last = self.last_score.lock().unwrap();
            if score > *last && *last != 0.0 && *last != f32::MAX {
                self.failed.store(true, Ordering::SeqCst);
            }
            *last = score;
            true
        }
    }

    impl ProgressCallback for &ScoreCheckCallback {
        fn on_progress(&self, epoch: usize, score: f32, layout: &[KeyCode], ips: f32) -> bool {
            (**self).on_progress(epoch, score, layout, ips)
        }
    }

    let callback = ScoreCheckCallback {
        last_score: std::sync::Mutex::new(f32::MAX),
        failed: AtomicBool::new(false),
    };

    let config = AnnealingConfig::new(1000, 0.0, 0.0, 42, 1000, 0, 1.0).unwrap();
    let mut optimizer = Optimizer::new(
        &engine,
        config,
        mutation,
        acceptance,
        crate::supervisor::traits::RealTimeKeeper,
    );

    optimizer.run(None, &callback).unwrap();
    assert!(!callback.failed.load(Ordering::SeqCst), "Score increased during zero-temperature annealing!");
}

#[test]
fn test_annealing_edge_cases() {
    let engine = setup_test_engine(2);

    // 1. Seed = 0 (Entropy)
    let config_entropy = AnnealingConfig::new(10, 1.0, 0.1, 0, 10, 0, 1.0).unwrap();
    let mut opt_entropy = Optimizer::new(
        &engine,
        config_entropy,
        GroupMutation { unlocked_indices: vec![0, 1] },
        CoolingAnnealing,
        crate::supervisor::traits::RealTimeKeeper,
    );
    opt_entropy.run(None, crate::NoOpCallback).unwrap();

    // 2. Steps = 0
    assert!(AnnealingConfig::new(0, 1.0, 0.1, 42, 10, 0, 1.0).is_err());

    // 3. Fast cooling
    let config_fast = AnnealingConfig::new(100, 1e-9, 1e-20, 42, 10, 0, 1.0).unwrap();
    let mut opt_fast = Optimizer::new(
        &engine,
        config_fast,
        GroupMutation { unlocked_indices: vec![0, 1] },
        CoolingAnnealing,
        crate::supervisor::traits::RealTimeKeeper,
    );
    opt_fast.run(None, crate::NoOpCallback).unwrap();
}

// --- Shadow Execution Tests ---

struct OracleCallback {
    keyboard: Arc<Keyboard>,
    corpus: Arc<Corpus>,
    rubric: Arc<Rubric>,
}

impl ProgressCallback for OracleCallback {
    fn on_progress(&self, step: usize, score: f32, layout_keys: &[KeyCode], _ips: f32) -> bool {
        let layout = Layout::new_unchecked(layout_keys.to_vec());
        let reference_score = keyforge_physics::verify::DeterministicScorer::score(
            &self.keyboard, 
            &self.corpus, 
            &self.rubric, 
            &layout, 
            &[]
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

#[test]
fn test_oracle_pattern_match() {
    let keyboard = Arc::new(create_mock_keyboard());
    let corpus = Arc::new(create_mock_corpus());
    let rubric = Arc::new(Rubric::default());
    
    let config = SearchConfig::Annealing {
        steps: 2000, start_temp: 10.0, end_temp: 0.1, seed: 42,
        patience: 100, reheats: 0, reheat_factor: 1.0,
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

    // This should now unwrap safely
    let result = optimize_with_callback(&req, callback).unwrap();

    let final_reference = keyforge_physics::verify::DeterministicScorer::score(
        &req.keyboard, &req.corpus, &req.rubric, &result.layout, &[]
    );
    
    assert!((result.score - final_reference).abs() < 1e-4);
}

// --- Singularity & Error Tests ---

#[test]
fn test_singularity_zero_temp_execution() {
    let keys: Vec<_> = (0..2)
        .map(|i| KeyNode {
            index: i,
            label: format!("k{}", i),
            hand: HandIndex(0),
            finger: FingerIndex(0),
            row: RowIndex(0),
            col: ColIndex(i as i8),
            x: i as f32,
            y: 0.0,
            is_home: false,
            ..Default::default()
        })
        .collect();
    let kb = Keyboard::new(keys, 0).unwrap();
    let engine = ScoringEngine::new(&kb, &Corpus::default(), &Rubric::default(), &[]).unwrap();
    
    let config = AnnealingConfig::new(100, 0.0, 0.0, 42, 10, 0, 1.0).unwrap();
    let mut optimizer = Optimizer::new(
        &engine,
        config,
        GroupMutation { unlocked_indices: vec![0, 1] },
        CoolingAnnealing,
        crate::supervisor::traits::RealTimeKeeper,
    );

    let result = optimizer.run(None, crate::NoOpCallback).unwrap();
    assert_eq!(result.keys.len(), 2);
}

#[test]
#[should_panic(expected = "Start temp must be > 0 to enable reheating")]
fn test_singularity_reheat_validation() {
    AnnealingConfig::new(100, 0.0, 0.0, 42, 10, 1, 1.0).unwrap();
}

#[test]
fn test_error_on_missing_pin() {
    let keys = vec![
        KeyNode { index: 0, label: "k0".into(), hand: HandIndex(0), finger: FingerIndex(0), row: RowIndex(0), col: ColIndex(0), x: 0.0, y: 0.0, is_home: false, ..Default::default() },
        KeyNode { index: 1, label: "k1".into(), hand: HandIndex(0), finger: FingerIndex(1), row: RowIndex(0), col: ColIndex(1), x: 1.0, y: 0.0, is_home: false, ..Default::default() },
    ];
    let kb = Arc::new(Keyboard::new(keys, 0).unwrap());
    let corpus = Arc::new(Corpus::default());
    let rubric = Arc::new(Rubric::default());
    
    let config = SearchConfig::Annealing {
        steps: 10, start_temp: 1.0, end_temp: 0.1, seed: 42,
        patience: 10, reheats: 0, reheat_factor: 1.0,
    };

    let pinned = vec![Some(KeyCode(99)), None];

    let req = EngineRequest {
        keyboard: kb, corpus, rubric, config,
        initial_layout: None, pinned_keys: pinned, cost_overrides: vec![],
    };

    let result = crate::optimize(&req);
    assert!(result.is_err());
    match result {
        Err(EvolutionError::Config(msg)) => assert!(msg.contains("Pinned key 99 not found")),
        _ => panic!("Expected Config error"),
    }
}

// --- Coverage & Edge Cases handled in one go ---

struct StagnantMutation;
impl MutationOperator for StagnantMutation {
    fn propose(
        &self,
        _engine: &ScoringEngine,
        _layout: &Layout,
        _pos_map: &[u16],
        _rng: &mut impl rand::Rng,
    ) -> Result<Option<MutationProposal>, EvolutionError> {
        Ok(Some(MutationProposal {
            delta: 1000,
            action: MutationAction::Swap(KeyIndex(0), KeyIndex(1)),
        }))
    }
}

#[test]
fn test_reheat_exhaustion() {
    let (kb, cp, rb) = setup_env_integrated();
    let engine = ScoringEngine::new(&kb, &cp, &rb, &[]).unwrap();
    let config = AnnealingConfig::new(100, 100.0, 0.1, 42, 2, 2, 2.0).unwrap();
    let mut optimizer = Optimizer::new(
        &engine, config, StagnantMutation, CoolingAnnealing, crate::supervisor::traits::RealTimeKeeper,
    );
    optimizer.run(None, crate::NoOpCallback).unwrap();
}
