use keyforge_evolution::supervisor::AnnealingConfig;
use keyforge_evolution::supervisor::strategies::{CoolingAnnealing, GroupMutation};
use keyforge_evolution::supervisor::traits::{MutationAction, MutationOperator, MutationProposal};
use keyforge_evolution::supervisor::Optimizer;
use keyforge_evolution::ProgressCallback;
use keyforge_model::{Corpus, KeyNode, Keyboard, Layout, Rubric, SearchConfig, types::{HandIndex, FingerIndex, RowIndex, ColIndex, KeyCode}};
use keyforge_physics::ScoringEngine;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_monotonicity_zero_temp() {
        let engine = setup_test_engine(30);
        let mutation = GroupMutation {
            unlocked_indices: (0..30).collect(),
        };
        let acceptance = CoolingAnnealing;

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
            keyforge_evolution::supervisor::traits::RealTimeKeeper,
        );

        optimizer.run(None, &callback);
        assert!(
            !callback.failed.load(Ordering::SeqCst),
            "Score increased during zero-temperature annealing!"
        );
    }

    #[test]
    fn test_state_integrity_after_reheat() {
        let engine = setup_test_engine(30);
        let mutation = GroupMutation {
            unlocked_indices: (0..30).collect(),
        };
        let acceptance = CoolingAnnealing;

        let config = AnnealingConfig::new(100, 1.0, 0.1, 42, 5, 1, 10.0).unwrap();
        let mut optimizer = Optimizer::new(
            &engine,
            config,
            mutation,
            acceptance,
            keyforge_evolution::supervisor::traits::RealTimeKeeper,
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
        let config_entropy = AnnealingConfig::new(10, 1.0, 0.1, 0, 10, 0, 1.0).unwrap();
        let mut opt_entropy = Optimizer::new(
            &engine,
            config_entropy,
            GroupMutation {
                unlocked_indices: vec![0, 1],
            },
            CoolingAnnealing,
            keyforge_evolution::supervisor::traits::RealTimeKeeper,
        );
        opt_entropy.run(None, keyforge_evolution::NoOpCallback);

        // 2. Steps = 0
        // AnnealingConfig::new should fail for steps=0
        assert!(AnnealingConfig::new(0, 1.0, 0.1, 42, 10, 0, 1.0).is_err());

        // 3. Fast cooling to hit zero-temp clamp (temp < 1e-10)
        let config_fast = AnnealingConfig::new(100, 1e-9, 1e-20, 42, 10, 0, 1.0).unwrap();
        let mut opt_fast = Optimizer::new(
            &engine,
            config_fast,
            GroupMutation {
                unlocked_indices: vec![0, 1],
            },
            CoolingAnnealing,
            keyforge_evolution::supervisor::traits::RealTimeKeeper,
        );
        opt_fast.run(None, keyforge_evolution::NoOpCallback);
    }

    #[test]
    fn test_progress_reporting_loop() {
        let engine = setup_test_engine(2);
        let mutation = GroupMutation {
            unlocked_indices: vec![0, 1],
        };
        let acceptance = CoolingAnnealing;

        let calls = Arc::new(AtomicUsize::new(0));
        struct ReportingCallback(Arc<AtomicUsize>);
        impl ProgressCallback for ReportingCallback {
            fn on_progress(&self, _epoch: usize, _score: f32, _layout: &[KeyCode], _ips: f32) -> bool {
                self.0.fetch_add(1, Ordering::SeqCst);
                true
            }
        }

        let config = AnnealingConfig::new(2100, 1.0, 0.1, 42, 2100, 0, 1.0).unwrap();
        let mut opt = Optimizer::new(
            &engine,
            config,
            mutation,
            acceptance,
            keyforge_evolution::supervisor::traits::RealTimeKeeper,
        );
        opt.run(None, ReportingCallback(calls.clone()));

        assert!(
            calls.load(Ordering::SeqCst) >= 2,
            "Progress callback not hit enough times!"
        );
    }

    #[test]
    fn test_optimizer_callback_break() {
        let engine = setup_test_engine(2);
        let mutation = GroupMutation {
            unlocked_indices: vec![0, 1],
        };
        let acceptance = CoolingAnnealing;

        struct BreakCallback;
        impl ProgressCallback for BreakCallback {
            fn on_progress(&self, _epoch: usize, _score: f32, _layout: &[KeyCode], _ips: f32) -> bool {
                false // Terminate immediately
            }
        }

        // report_interval is (total_steps / 100).max(1000)
        // So 2000 steps means report_interval = 1000.
        let config = AnnealingConfig::new(1001, 1.0, 0.1, 42, 2000, 0, 1.0).unwrap();
        let mut opt = Optimizer::new(
            &engine,
            config,
            mutation,
            acceptance,
            keyforge_evolution::supervisor::traits::RealTimeKeeper,
        );
        let best = opt.run(None, BreakCallback);
        assert_eq!(best.keys.len(), 2);
    }

    #[test]
    fn test_saturation_and_ips_branches() {
        let engine = setup_test_engine(30);

        // Mock mutation that returns a positive delta to test saturation
        struct SaturatingMutation;
        impl MutationOperator for SaturatingMutation {
            fn propose(
                &self,
                _engine: &ScoringEngine,
                _layout: &Layout,
                _pos_map: &[u16],
                _rng: &mut impl rand::Rng,
            ) -> Option<MutationProposal> {
                Some(MutationProposal {
                    delta: i64::MAX - 10,
                    action: MutationAction::Swap(
                        keyforge_model::KeyIndex(0),
                        keyforge_model::KeyIndex(1),
                    ),
                })
            }
        }

        let config = AnnealingConfig::new(1001, 1.0, 0.1, 42, 1000, 0, 1.0).unwrap();
        let mut opt = Optimizer::new(
            &engine,
            config,
            SaturatingMutation,
            CoolingAnnealing,
            keyforge_evolution::supervisor::traits::RealTimeKeeper,
        );
        opt.run(None, keyforge_evolution::NoOpCallback);
    }

    #[test]
    fn test_trigram_path_forcer() {
        let keys: Vec<_> = (0..30)
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
        for i in 0..28 {
            corpus
                .trigrams
                .push((i as u16, (i + 1) as u16, (i + 2) as u16, 100));
        }
        let engine = ScoringEngine::new(&kb, &corpus, &Rubric::default(), &[]).unwrap();
        let config = AnnealingConfig::new(10, 1.0, 0.1, 42, 10, 0, 1.0).unwrap();
        let mut opt = Optimizer::new(
            &engine,
            config,
            GroupMutation {
                unlocked_indices: (0..30).collect(),
            },
            CoolingAnnealing,
            keyforge_evolution::supervisor::traits::RealTimeKeeper,
        );
        opt.run(None, keyforge_evolution::NoOpCallback);
    }

    #[test]
    fn test_legacy_api_coverage() {
        let keys: Vec<_> = (0..30)
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
        let kb = Arc::new(Keyboard::new(keys, 1).unwrap());
        let corpus = Arc::new(Corpus::default());
        let rubric = Arc::new(Rubric::default());
        let engine = Arc::new(ScoringEngine::new(&kb, &corpus, &rubric, &[]).unwrap());

        let req_config = SearchConfig::Annealing {
            steps: 5,
            start_temp: 1.0,
            end_temp: 0.1,
            seed: 42,
            patience: 10,
            reheats: 0,
            reheat_factor: 1.0,
        };
        keyforge_evolution::evolve(engine, &req_config, keyforge_evolution::NoOpCallback);
    }
}
