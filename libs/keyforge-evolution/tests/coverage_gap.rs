use keyforge_evolution::supervisor::AnnealingConfig;
use keyforge_evolution::supervisor::traits::{MutationAction, MutationOperator, MutationProposal};
use keyforge_evolution::{evolve, optimize, optimize_with_callback, ProgressCallback};
use keyforge_model::{Corpus, KeyIndex, KeyNode, Keyboard, Layout, Rubric, SearchConfig, KeyCode};
use keyforge_physics::{EngineRequest, ScoringEngine};
use std::sync::Arc;

// --- Mocks ---

struct StagnantMutation;
impl MutationOperator for StagnantMutation {
    fn propose(
        &self,
        _engine: &ScoringEngine,
        _layout: &Layout,
        _pos_map: &[u16],
        _rng: &mut impl rand::Rng,
    ) -> Option<MutationProposal> {
        // Return a positive delta (bad move) to verify reheat logic handles it
        Some(MutationProposal {
            delta: 1000,
            action: MutationAction::Swap(KeyIndex(0), KeyIndex(1)),
        })
    }
}


struct CountingCallback {
    counter: Arc<std::sync::atomic::AtomicUsize>,
    limit: usize,
}

impl ProgressCallback for CountingCallback {
    fn on_progress(&self, _step: usize, _score: f32, _layout: &[KeyCode], _ips: f32) -> bool {
        let val = self.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        val < self.limit
    }
}

use keyforge_model::types::{HandIndex, FingerIndex, RowIndex, ColIndex};

fn setup_env() -> (Arc<Keyboard>, Arc<Corpus>, Arc<Rubric>) {
    let keys = vec![
        KeyNode {
            index: 0,
            label: "k0".to_string(),
            hand: HandIndex(0),
            finger: FingerIndex(1),
            row: RowIndex(0),
            col: ColIndex(0),
            x: 0.0,
            y: 0.0,
            is_home: true,
            ..Default::default()
        },
        KeyNode {
            index: 1,
            label: "k1".to_string(),
            hand: HandIndex(0),
            finger: FingerIndex(2),
            row: RowIndex(0),
            col: ColIndex(1),
            x: 1.0,
            y: 0.0,
            is_home: true,
            ..Default::default()
        },
        KeyNode {
            index: 2,
            label: "k2".to_string(),
            hand: HandIndex(0),
            finger: FingerIndex(3),
            row: RowIndex(0),
            col: ColIndex(2),
            x: 2.0,
            y: 0.0,
            is_home: true,
            ..Default::default()
        },
    ];
    (
        Arc::new(Keyboard::new(keys, 0).unwrap()),
        Arc::new(Corpus::default()),
        Arc::new(Rubric::default()),
    )
}

#[test]
fn test_force_reheat_logic() {
    let (kb, cp, rb) = setup_env();
    let engine = ScoringEngine::new(&kb, &cp, &rb, &[]).unwrap();
    
    // Config: Short patience, high reheats
    let steps = 100;
    let patience = 5;
    let reheats = 2;
    
    let config = AnnealingConfig::new(steps, 100.0, 0.1, 42, patience, reheats, 2.0).unwrap();

    let mut optimizer = keyforge_evolution::supervisor::Optimizer::new(
        &engine,
        config,
        StagnantMutation, // Will never improve
        keyforge_evolution::supervisor::strategies::CoolingAnnealing,
        keyforge_evolution::supervisor::traits::RealTimeKeeper,
    );

    let _result = optimizer.run(None, keyforge_evolution::NoOpCallback);
}

#[test]
fn test_legacy_optimize_entry_point() {
    let (kb, cp, rb) = setup_env();
    
    let req = EngineRequest {
        keyboard: kb,
        corpus: cp,
        rubric: rb,
        config: SearchConfig::Annealing {
            steps: 10,
            start_temp: 10.0,
            end_temp: 1.0,
            seed: 123,
            patience: 100,
            reheats: 0,
            reheat_factor: 1.0,
        },
        initial_layout: None,
        pinned_keys: vec![],
        cost_overrides: vec![],
    };

    let result = optimize(&req);
    assert!(result.score >= 0.0);
}

#[test]
fn test_legacy_optimize_full_options() {
    let (kb, cp, rb) = setup_env();
    
    // Test with pinned keys and initial layout
    let req = EngineRequest {
        keyboard: kb,
        corpus: cp,
        rubric: rb,
        config: SearchConfig::Annealing {
            steps: 10,
            start_temp: 10.0,
            end_temp: 1.0,
            seed: 123,
            patience: 100,
            reheats: 0,
            reheat_factor: 1.0,
        },
        initial_layout: Some(Layout::new_unchecked(vec![KeyCode(1), KeyCode(0), KeyCode(2)])),
        pinned_keys: vec![Some(KeyCode(1)), None], // Pin index 0 to keycode 1
        cost_overrides: vec![],
    };

    let result = optimize(&req);
    assert_eq!(result.layout.keys[0], KeyCode(1)); // Should respect pin
}

#[test]
fn test_optimize_with_callback_termination() {
    let (kb, cp, rb) = setup_env();
    
    // Set steps high enough to trigger reporting multiple times.
    // Interval = max(steps/100, 1000).
    // If steps=1_000_000, interval=10,000.
    // Let's use steps=5000, interval=1000.
    // Reports at 1000, 2000, 3000, 4000.
    let req = EngineRequest {
        keyboard: kb,
        corpus: cp,
        rubric: rb,
        config: SearchConfig::Annealing {
            steps: 5000, 
            start_temp: 10.0,
            end_temp: 1.0,
            seed: 123,
            patience: 100,
            reheats: 0,
            reheat_factor: 1.0,
        },
        initial_layout: None,
        pinned_keys: vec![],
        cost_overrides: vec![],
    };

    let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    // Limit = 1. Should return true (0 < 1) on first call, then false (1 < 1) on second call.
    // First call at 1000. Returns true.
    // Second call at 2000. Returns false. Break.
    // Total calls should be 2.
    let callback = CountingCallback { 
        counter: counter.clone(), 
        limit: 1 
    };
    
    let result = optimize_with_callback(&req, callback);
    
    assert!(result.score >= 0.0);
    assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 2, "Should have called callback exactly twice before breaking");
}

#[test]
fn test_optimize_with_callback_early_termination() {
    let (kb, cp, rb) = setup_env();
    
    let req = EngineRequest {
        keyboard: kb,
        corpus: cp,
        rubric: rb,
        config: SearchConfig::Annealing {
            steps: 1000, 
            start_temp: 10.0,
            end_temp: 1.0,
            seed: 123,
            patience: 100,
            reheats: 0,
            reheat_factor: 1.0,
        },
        initial_layout: None,
        pinned_keys: vec![],
        cost_overrides: vec![],
    };

    let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    // Steps=1000. Report interval=1000.
    // Call 1 at 1000. Returns true (val=0 < limit=1).
    // Finishes loop.
    let callback = CountingCallback { counter: counter.clone(), limit: 1 };
    let result = optimize_with_callback(&req, callback);
    
    assert!(result.score >= 0.0);
}

#[test]
fn test_evolve_api_direct() {
    let (kb, cp, rb) = setup_env();
    let engine = Arc::new(ScoringEngine::new(&kb, &cp, &rb, &[]).unwrap());
    
    let config = SearchConfig::Annealing {
        steps: 10,
        start_temp: 10.0,
        end_temp: 1.0,
        seed: 123,
        patience: 100,
        reheats: 0,
        reheat_factor: 1.0,
    };

    let result = evolve(engine, &config, keyforge_evolution::NoOpCallback);
    assert!(result.score >= 0.0);
}

#[test]
fn test_reheat_exhaustion() {
    let (kb, cp, rb) = setup_env();
    let engine = ScoringEngine::new(&kb, &cp, &rb, &[]).unwrap();
    
    let steps = 100;
    let patience = 2;
    let reheats = 2;
    
    let config = AnnealingConfig::new(steps, 100.0, 0.1, 42, patience, reheats, 2.0).unwrap();

    let mut optimizer = keyforge_evolution::supervisor::Optimizer::new(
        &engine,
        config,
        StagnantMutation,
        keyforge_evolution::supervisor::strategies::CoolingAnnealing,
        keyforge_evolution::supervisor::traits::RealTimeKeeper,
    );

    let _result = optimizer.run(None, keyforge_evolution::NoOpCallback);
}

#[test]
fn test_initial_layout_application() {
    let (kb, cp, rb) = setup_env();
    
    let initial = Layout::new_unchecked(vec![KeyCode(2), KeyCode(1), KeyCode(0)]);
    
    let req = EngineRequest {
        keyboard: kb,
        corpus: cp,
        rubric: rb,
        config: SearchConfig::Annealing {
            steps: 5,
            start_temp: 0.0,
            end_temp: 0.0,
            seed: 123,
            patience: 100,
            reheats: 0,
            reheat_factor: 1.0,
        },
        initial_layout: Some(initial.clone()),
        pinned_keys: vec![],
        cost_overrides: vec![],
    };

    let result = optimize(&req);
    assert_eq!(result.layout.keys.len(), 3);
}

#[test]
fn test_pinned_key_injection() {
    let (kb, cp, rb) = setup_env();
    
    let pinned = vec![Some(KeyCode(99)), None, None];
    
    let req = EngineRequest {
        keyboard: kb,
        corpus: cp,
        rubric: rb,
        config: SearchConfig::Annealing {
            steps: 10,
            start_temp: 10.0,
            end_temp: 1.0,
            seed: 123,
            patience: 100,
            reheats: 0,
            reheat_factor: 1.0,
        },
        initial_layout: None,
        pinned_keys: pinned,
        cost_overrides: vec![],
    };

    // This test expects injection, but new logic requires permutation integrity.
    // If 99 is not in the initial layout (0,1,2), it should panic.
    // We must update the test to provide an initial layout containing 99.
    // Or update the test expectation to fail.
    // Let's update the test to be valid under new doctrine.
    
    // Actually, let's skip this test or modify it to use a valid initial layout.
    // Since we can't easily modify the test logic without rewriting the whole file,
    // and I am rewriting the whole file, I will fix it here.
    
    // New logic: Provide initial layout with 99.
    let initial = Layout::new_unchecked(vec![KeyCode(99), KeyCode(1), KeyCode(2)]);
    let req_valid = EngineRequest {
        initial_layout: Some(initial),
        ..req
    };

    let result = optimize(&req_valid);
    assert_eq!(result.layout.keys[0], KeyCode(99));
}

#[test]
fn test_low_temp_clamping() {
    let (kb, cp, rb) = setup_env();
    let engine = Arc::new(ScoringEngine::new(&kb, &cp, &rb, &[]).unwrap());
    
    let config = SearchConfig::Annealing {
        steps: 100,
        start_temp: 1e-11, 
        end_temp: 1e-20,
        seed: 42,
        patience: 100,
        reheats: 0,
        reheat_factor: 1.0,
    };

    let result = evolve(engine, &config, keyforge_evolution::NoOpCallback);
    assert!(result.score >= 0.0);
}

#[test]
fn test_ips_calculation() {
    let (kb, cp, rb) = setup_env();
    let engine = Arc::new(ScoringEngine::new(&kb, &cp, &rb, &[]).unwrap());
    
    let config = SearchConfig::Annealing {
        steps: 2000, 
        start_temp: 10.0,
        end_temp: 1.0,
        seed: 42,
        patience: 100,
        reheats: 0,
        reheat_factor: 1.0,
    };

    struct SleepyCallback;
    impl ProgressCallback for SleepyCallback {
        fn on_progress(&self, _step: usize, _score: f32, _layout: &[KeyCode], ips: f32) -> bool {
            std::thread::sleep(std::time::Duration::from_millis(1));
            assert!(ips >= 0.0);
            true
        }
    }

    evolve(engine, &config, SleepyCallback);
}

#[test]
fn test_pinned_key_swap() {
    let (kb, cp, rb) = setup_env();
    
    // Default layout keys: [0, 1, 2]
    // Pin key '2' to position '0'.
    // Logic should find '2' at pos 2 and swap it with pos 0.
    // Result: [2, 1, 0]
    let pinned = vec![Some(KeyCode(2)), None, None];
    
    let req = EngineRequest {
        keyboard: kb,
        corpus: cp,
        rubric: rb,
        config: SearchConfig::Annealing {
            steps: 10,
            start_temp: 10.0,
            end_temp: 1.0,
            seed: 123,
            patience: 100,
            reheats: 0,
            reheat_factor: 1.0,
        },
        initial_layout: None,
        pinned_keys: pinned,
        cost_overrides: vec![],
    };

    let result = optimize(&req);
    // Key 2 should be at index 0
    assert_eq!(result.layout.keys[0], KeyCode(2));
    // Key 0 should be swapped to index 2
    assert_eq!(result.layout.keys[2], KeyCode(0));
}

#[test]
fn test_insufficient_unlocked_keys() {
    let (kb, cp, rb) = setup_env();
    
    // Pin 2 out of 3 keys. Only 1 unlocked.
    // GroupMutation should return None (lines 56-57 in strategies.rs)
    let pinned = vec![Some(KeyCode(0)), Some(KeyCode(1)), None];
    
    let req = EngineRequest {
        keyboard: kb,
        corpus: cp,
        rubric: rb,
        config: SearchConfig::Annealing {
            steps: 100,
            start_temp: 10.0,
            end_temp: 1.0,
            seed: 123,
            patience: 100,
            reheats: 0,
            reheat_factor: 1.0,
        },
        initial_layout: None,
        pinned_keys: pinned,
        cost_overrides: vec![],
    };

    let result = optimize(&req);
    // Should verify that the result is valid (no panic) 
    // and keys are respected.
    assert_eq!(result.layout.keys[0], KeyCode(0));
    assert_eq!(result.layout.keys[1], KeyCode(1));
}
