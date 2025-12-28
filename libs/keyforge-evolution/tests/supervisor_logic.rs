use keyforge_evolution::ProgressCallback;
use keyforge_evolution::supervisor::Optimizer;
use keyforge_evolution::supervisor::strategies::{CoolingAnnealing, GroupMutation};
use keyforge_evolution::supervisor::traits::{MutationOperator, MutationProposal, MutationAction};
use keyforge_model::{Corpus, KeyNode, Keyboard, Layout, Rubric, SearchConfig};
use keyforge_physics::ScoringEngine;
use std::sync::atomic::{AtomicBool, Ordering, AtomicUsize};
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;

    struct ScoreCheckCallback {
        last_score: std::sync::Mutex<f32>,
        failed: AtomicBool,
    }

    impl ProgressCallback for ScoreCheckCallback {
        fn on_progress(&self, _epoch: usize, score: f32, _layout: &[u16], _ips: f32) -> bool {
            let mut last = self.last_score.lock().unwrap();
            if score > *last && *last != 0.0 && *last != f32::MAX {
                 self.failed.store(true, Ordering::SeqCst);
            }
            *last = score;
            true
        }
    }

    impl ProgressCallback for &ScoreCheckCallback {
        fn on_progress(&self, epoch: usize, score: f32, layout: &[u16], ips: f32) -> bool {
            (**self).on_progress(epoch, score, layout, ips)
        }
    }

    fn setup_test_engine(size: usize) -> ScoringEngine {
        let keys: Vec<_> = (0..size).map(|i| KeyNode {
            id: i,
            label: format!("k{}", i),
            hand: (i % 2) as u8,
            finger: (i % 5) as u8,
            row: (i / 10) as i8,
            col: (i % 10) as i8,
            x: (i % 10) as f32,
            y: (i / 10) as f32,
            is_home: false,
        }).collect();
        let kb = Keyboard::new(keys, 1);
        let mut corpus = Corpus::default();
        for i in 0..size {
            corpus.char_freqs[i] = (i * 10) as u32;
            if i + 1 < size {
                corpus.bigrams.push((i as u16, (i + 1) as u16, 100));
            }
        }
        ScoringEngine::new(&kb, &corpus, &Rubric::default(), &[])
    }

    #[test]
    fn test_monotonicity_zero_temp() {
        let engine = setup_test_engine(30);
        let mutation = GroupMutation { unlocked_indices: (0..30).collect() };
        let acceptance = CoolingAnnealing;
        
        let callback = ScoreCheckCallback {
            last_score: std::sync::Mutex::new(f32::MAX),
            failed: AtomicBool::new(false),
        };

        let mut optimizer = Optimizer::new(
            &engine,
            1000,
            0.0,       // ZERO TEMP
            0.0,
            42,
            mutation,
            acceptance,
            1000,
            0,
            1.0,
        );

        optimizer.run(None, &callback);
        assert!(!callback.failed.load(Ordering::SeqCst), "Score increased during zero-temperature annealing!");
    }

    #[test]
    fn test_state_integrity_after_reheat() {
        let engine = setup_test_engine(30);
        let mutation = GroupMutation { unlocked_indices: (0..30).collect() };
        let acceptance = CoolingAnnealing;
        
        let mut optimizer = Optimizer::new(
            &engine,
            100,
            1.0,
            0.1,
            42,
            mutation,
            acceptance,
            5,         // Low patience to trigger reheat
            1,         // 1 reheat
            10.0,      // High reheat factor
        );

        let final_layout = optimizer.run(None, keyforge_evolution::NoOpCallback);
        
        // Final layout should be valid (no duplicate keys)
        let mut seen = std::collections::HashSet::new();
        for &k in &final_layout.keys {
            assert!(seen.insert(k), "Duplicate key {} in final layout!", k);
        }
        assert_eq!(final_layout.keys.len(), 30);
    }

    #[test]
    fn test_annealing_edge_cases() {
        let engine = setup_test_engine(2);

        // 1. Seed = 0 (Entropy)
        let mut opt_entropy = Optimizer::new(&engine, 10, 1.0, 0.1, 0, GroupMutation { unlocked_indices: vec![0, 1] }, CoolingAnnealing, 10, 0, 1.0);
        opt_entropy.run(None, keyforge_evolution::NoOpCallback);

        // 2. Steps = 0
        let mut opt_zero_steps = Optimizer::new(&engine, 0, 1.0, 0.1, 42, GroupMutation { unlocked_indices: vec![0, 1] }, CoolingAnnealing, 10, 0, 1.0);
        opt_zero_steps.run(None, keyforge_evolution::NoOpCallback);

        // 3. Fast cooling to hit zero-temp clamp (temp < 1e-10)
        let mut opt_fast = Optimizer::new(&engine, 100, 1e-9, 1e-20, 42, GroupMutation { unlocked_indices: vec![0, 1] }, CoolingAnnealing, 10, 0, 1.0);
        opt_fast.run(None, keyforge_evolution::NoOpCallback);
    }

    #[test]
    fn test_progress_reporting_loop() {
        let engine = setup_test_engine(2);
        let mutation = GroupMutation { unlocked_indices: vec![0, 1] };
        let acceptance = CoolingAnnealing;

        let calls = Arc::new(AtomicUsize::new(0));
        struct ReportingCallback(Arc<AtomicUsize>);
        impl ProgressCallback for ReportingCallback {
            fn on_progress(&self, _epoch: usize, _score: f32, _layout: &[u16], _ips: f32) -> bool {
                self.0.fetch_add(1, Ordering::SeqCst);
                true
            }
        }

        let mut opt = Optimizer::new(&engine, 2100, 1.0, 0.1, 42, mutation, acceptance, 2100, 0, 1.0);
        opt.run(None, ReportingCallback(calls.clone()));

        assert!(calls.load(Ordering::SeqCst) >= 2, "Progress callback not hit enough times!");
    }

    #[test]
    fn test_optimizer_callback_break() {
        let engine = setup_test_engine(2);
        let mutation = GroupMutation { unlocked_indices: vec![0, 1] };
        let acceptance = CoolingAnnealing;

        struct BreakCallback;
        impl ProgressCallback for BreakCallback {
            fn on_progress(&self, _epoch: usize, _score: f32, _layout: &[u16], _ips: f32) -> bool {
                false // Terminate immediately
            }
        }

        // report_interval is (total_steps / 100).max(1000)
        // So 2000 steps means report_interval = 1000.
        let mut opt = Optimizer::new(&engine, 1001, 1.0, 0.1, 42, mutation, acceptance, 2000, 0, 1.0);
        let best = opt.run(None, BreakCallback);
        assert_eq!(best.keys.len(), 2);
    }

    #[test]
    fn test_saturation_and_ips_branches() {
        let engine = setup_test_engine(30);
        
        // Mock mutation that returns a positive delta to test saturation
        struct SaturatingMutation;
        impl MutationOperator for SaturatingMutation {
            fn propose(&self, _engine: &ScoringEngine, _layout: &Layout, _pos_map: &[u16], _rng: &mut impl rand::Rng) -> Option<MutationProposal> {
                Some(MutationProposal {
                    delta: i64::MAX - 10,
                    action: MutationAction::Swap(0, 1),
                })
            }
        }
        
        let mut opt = Optimizer::new(&engine, 1001, 1.0, 0.1, 42, SaturatingMutation, CoolingAnnealing, 1000, 0, 1.0);
        opt.run(None, keyforge_evolution::NoOpCallback);
    }

    #[test]
    fn test_trigram_path_forcer() {
        let keys: Vec<_> = (0..30).map(|i| KeyNode {
            id: i, label: format!("k{}", i), hand: (i % 2) as u8, finger: (i % 5) as u8,
            row: (i / 10) as i8, col: (i % 10) as i8, x: (i % 10) as f32, y: (i / 10) as f32, is_home: false,
        }).collect();
        let kb = Keyboard::new(keys, 1);
        let mut corpus = Corpus::default();
        for i in 0..28 {
            corpus.trigrams.push((i as u16, (i+1) as u16, (i+2) as u16, 100));
        }
        let engine = ScoringEngine::new(&kb, &corpus, &Rubric::default(), &[]);
        let mut opt = Optimizer::new(&engine, 10, 1.0, 0.1, 42, GroupMutation { unlocked_indices: (0..30).collect() }, CoolingAnnealing, 10, 0, 1.0);
        opt.run(None, keyforge_evolution::NoOpCallback);
    }

    #[test]
    fn test_legacy_api_coverage() {
        let keys: Vec<_> = (0..30).map(|i| KeyNode {
            id: i, label: format!("k{}", i), hand: (i % 2) as u8, finger: (i % 5) as u8,
            row: (i / 10) as i8, col: (i % 10) as i8, x: (i % 10) as f32, y: (i / 10) as f32, is_home: false,
        }).collect();
        let kb = Arc::new(Keyboard::new(keys, 1));
        let corpus = Arc::new(Corpus::default());
        let rubric = Arc::new(Rubric::default());
        let engine = Arc::new(ScoringEngine::new(&kb, &corpus, &rubric, &[]));
        
        let req_config = SearchConfig::Annealing {
            steps: 5, start_temp: 1.0, end_temp: 0.1, seed: 42, patience: 10, reheats: 0, reheat_factor: 1.0,
        };
        keyforge_evolution::evolve(engine, &req_config, keyforge_evolution::NoOpCallback);
    }
}
