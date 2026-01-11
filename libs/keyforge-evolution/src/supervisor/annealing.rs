// libs/keyforge-evolution/src/supervisor/annealing.rs

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

use super::state::SearchState;
use super::traits::{AcceptanceCriteria, MutationOperator, TimeKeeper};
use crate::errors::EvolutionError;
use crate::ProgressCallback;
use keyforge_model::{Layout, KeyCode};
use keyforge_physics::ScoringEngine;
use keyforge_model::constants::{SCORE_SCALE, TEMP_UNDERFLOW_THRESHOLD, DEFAULT_REPORT_DIVISOR, MIN_REPORT_INTERVAL};
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;

#[derive(Debug, Clone, Copy)]
pub struct AnnealingConfig {
    pub steps: usize,
    pub start_temp: f32,
    pub end_temp: f32,
    pub seed: u64,
    pub patience: usize,
    pub reheats: usize,
    pub reheat_factor: f32,
}

impl AnnealingConfig {
    pub fn new(
        steps: usize,
        start_temp: f32,
        end_temp: f32,
        seed: u64,
        patience: usize,
        reheats: usize,
        reheat_factor: f32,
    ) -> Result<Self, EvolutionError> {
        if steps == 0 {
            return Err(EvolutionError::Config("Steps must be > 0".into()));
        }
        if reheats > 0 && start_temp <= f32::EPSILON {
            return Err(EvolutionError::Config(
                "Start temp must be > 0 to enable reheating".into(),
            ));
        }
        if start_temp < 0.0 || end_temp < 0.0 {
            return Err(EvolutionError::Config(
                "Temperatures must be non-negative".into(),
            ));
        }
        if reheat_factor <= 0.0 {
            return Err(EvolutionError::Config(
                "Reheat factor must be > 0.0".into(),
            ));
        }
        Ok(Self {
            steps,
            start_temp,
            end_temp,
            seed,
            patience,
            reheats,
            reheat_factor,
        })
    }
}

pub struct Optimizer<'a, M: MutationOperator, A: AcceptanceCriteria, T: TimeKeeper> {
    engine: &'a ScoringEngine,
    config: AnnealingConfig,
    rng: Xoshiro256PlusPlus,
    mutation: M,
    acceptance: A,
    time_keeper: T,
}

impl<'a, M: MutationOperator, A: AcceptanceCriteria, T: TimeKeeper> Optimizer<'a, M, A, T> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        engine: &'a ScoringEngine,
        config: AnnealingConfig,
        mutation: M,
        acceptance: A,
        time_keeper: T,
    ) -> Self {
        let rng = if config.seed == 0 {
            Xoshiro256PlusPlus::from_rng(&mut rand::rng())
        } else {
            Xoshiro256PlusPlus::seed_from_u64(config.seed)
        };

        Self {
            engine,
            config,
            rng,
            mutation,
            acceptance,
            time_keeper,
        }
    }

    pub fn run<CB: ProgressCallback>(
        &mut self,
        initial_layout: Option<Layout>,
        callback: CB,
    ) -> Result<Layout, EvolutionError> {
        let layout = initial_layout.unwrap_or_else(|| {
            let keys: Vec<KeyCode> = (0..self.engine.key_count()).map(|i| KeyCode(i as u16)).collect();
            Layout::new_unchecked(keys)
        });

        let initial_score = self.engine.score_raw(&layout.keys)?;

        // INVARIANT: kani::assume(initial_score >= 0);
        let mut state = SearchState::new(layout, initial_score, self.config.start_temp)?;

        let cooling_rate = if self.config.steps > 0 && self.config.start_temp > f32::EPSILON {
            (self.config.end_temp / self.config.start_temp).powf(1.0 / self.config.steps as f32)
        } else {
            0.0
        };

        let report_interval = (self.config.steps / DEFAULT_REPORT_DIVISOR).max(MIN_REPORT_INTERVAL);
        let start_time = self.time_keeper.now();
        let mut last_report_time = start_time;
        let mut last_report_step = 0;

        let mut steps_since_improvement = 0;
        let mut reheats_left = self.config.reheats;

        for step in 0..self.config.steps {
            if let Some(proposal) = self.mutation.propose(
                self.engine,
                state.layout(),
                state.pos_map(),
                &mut self.rng,
            )? {
                if self.acceptance.should_accept(
                    proposal.delta,
                    state.temperature,
                    &mut self.rng,
                ) {
                    state.apply_mutation(proposal.action);

                    state.current_score = state.current_score.checked_add(proposal.delta).unwrap_or(
                        if proposal.delta > 0 {
                            i64::MAX
                        } else {
                            i64::MIN
                        },
                    );

                    if state.current_score < state.best_score {
                        state.update_best();
                        steps_since_improvement = 0;
                    } else {
                        steps_since_improvement += 1;
                    }
                } else {
                    steps_since_improvement += 1;
                }
            }

            // Reheating Logic
            if steps_since_improvement > self.config.patience && reheats_left > 0 {
                state.reheat_from_best(self.config.start_temp, self.config.reheat_factor);

                reheats_left -= 1;
                steps_since_improvement = 0;
            }

            state.temperature *= cooling_rate;
            if state.temperature < TEMP_UNDERFLOW_THRESHOLD {
                state.temperature = 0.0;
            }

            if step % report_interval == 0 {
                let now = self.time_keeper.now();
                let elapsed = self.time_keeper.elapsed(last_report_time).as_secs_f32();
                let steps_done = if step == 0 { 0 } else { step - last_report_step };

                let ips = if elapsed > 0.0 {
                    (steps_done as f32 / elapsed) / 1_000_000.0
                } else {
                    0.0
                };

                let score_f32 = state.best_score as f32 / SCORE_SCALE;
                if !callback.on_progress(step, score_f32, &state.best_layout().keys, ips) {
                    return Err(EvolutionError::Aborted);
                }

                if step > 0 {
                    last_report_time = now;
                    last_report_step = step;
                }
            }
        }

        Ok(state.best_layout().clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::supervisor::strategies::{CoolingAnnealing, GroupMutation};
    use crate::supervisor::traits::{MutationAction, MutationProposal};
    use keyforge_model::{Corpus, KeyNode, Keyboard, Rubric};
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
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
        ) -> Result<Option<MutationProposal>, EvolutionError> {
            Ok(Some(MutationProposal {
                delta: 1000,
                action: MutationAction::Swap(keyforge_model::KeyIndex(0), keyforge_model::KeyIndex(1)),
            }))
        }
    }

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

    use keyforge_model::types::{HandIndex, FingerIndex, RowIndex, ColIndex};

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
            corpus.char_freqs[i] = (i * 10) as u64; // FIX: Cast to u64
            if i + 1 < size {
                corpus.bigrams.push((i as u16, (i + 1) as u16, 100));
            }
        }
        ScoringEngine::new(&kb, &corpus, &Rubric::default(), &[]).unwrap()
    }

    #[test]
    fn test_force_reheat_logic() {
        let engine = setup_test_engine(2);
        let config = AnnealingConfig::new(100, 100.0, 0.1, 42, 5, 2, 2.0).unwrap();
        let mut optimizer = Optimizer::new(
            &engine,
            config,
            StagnantMutation,
            CoolingAnnealing,
            crate::supervisor::traits::RealTimeKeeper,
        );
        optimizer.run(None, crate::NoOpCallback).unwrap();
    }

    #[test]
    fn test_singularity_zero_temp_execution() {
        let engine = setup_test_engine(2);
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
    fn test_monotonicity_zero_temp() {
        let engine = setup_test_engine(30);
        let mutation = GroupMutation { unlocked_indices: (0..30).collect() };
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
            crate::supervisor::traits::RealTimeKeeper,
        );
        optimizer.run(None, &callback).unwrap();
        assert!(!callback.failed.load(Ordering::SeqCst), "Score increased during zero-temperature annealing!");
    }

    #[test]
    fn test_state_integrity_after_reheat() {
        let engine = setup_test_engine(30);
        let mutation = GroupMutation { unlocked_indices: (0..30).collect() };
        let acceptance = CoolingAnnealing;
        let config = AnnealingConfig::new(100, 1.0, 0.1, 42, 5, 1, 10.0).unwrap();
        let mut optimizer = Optimizer::new(
            &engine,
            config,
            mutation,
            acceptance,
            crate::supervisor::traits::RealTimeKeeper,
        );
        let final_layout = optimizer.run(None, crate::NoOpCallback).unwrap();
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

    #[test]
    fn test_progress_reporting_loop() {
        let engine = setup_test_engine(2);
        let mutation = GroupMutation { unlocked_indices: vec![0, 1] };
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
            crate::supervisor::traits::RealTimeKeeper,
        );
        opt.run(None, ReportingCallback(calls.clone())).unwrap();
        assert!(calls.load(Ordering::SeqCst) >= 2, "Progress callback not hit enough times!");
    }

    #[test]
    fn test_optimizer_callback_break() {
        let engine = setup_test_engine(2);
        let mutation = GroupMutation { unlocked_indices: vec![0, 1] };
        let acceptance = CoolingAnnealing;
        struct BreakCallback;
        impl ProgressCallback for BreakCallback {
            fn on_progress(&self, _epoch: usize, _score: f32, _layout: &[KeyCode], _ips: f32) -> bool {
                false
            }
        }
        let config = AnnealingConfig::new(1001, 1.0, 0.1, 42, 2000, 0, 1.0).unwrap();
        let mut opt = Optimizer::new(
            &engine,
            config,
            mutation,
            acceptance,
            crate::supervisor::traits::RealTimeKeeper,
        );
        let res = opt.run(None, BreakCallback);
        assert!(matches!(res, Err(EvolutionError::Aborted)));
    }

    #[test]
    fn test_saturation_and_ips_branches() {
        let engine = setup_test_engine(30);
        struct SaturatingMutation;
        impl MutationOperator for SaturatingMutation {
            fn propose(
                &self,
                _engine: &ScoringEngine,
                _layout: &Layout,
                _pos_map: &[u16],
                _rng: &mut impl rand::Rng,
            ) -> Result<Option<MutationProposal>, EvolutionError> {
                Ok(Some(MutationProposal {
                    delta: i64::MAX - 10,
                    action: MutationAction::Swap(keyforge_model::KeyIndex(0), keyforge_model::KeyIndex(1)),
                }))
            }
        }
        let config = AnnealingConfig::new(1001, 1.0, 0.1, 42, 1000, 0, 1.0).unwrap();
        let mut opt = Optimizer::new(
            &engine,
            config,
            SaturatingMutation,
            CoolingAnnealing,
            crate::supervisor::traits::RealTimeKeeper,
        );
        opt.run(None, crate::NoOpCallback).unwrap();
    }

    #[test]
    fn test_reheat_exhaustion() {
        let engine = setup_test_engine(2);
        let config = AnnealingConfig::new(100, 100.0, 0.1, 42, 2, 2, 2.0).unwrap();
        let mut optimizer = Optimizer::new(
            &engine, config, StagnantMutation, CoolingAnnealing, crate::supervisor::traits::RealTimeKeeper,
        );
        optimizer.run(None, crate::NoOpCallback).unwrap();
    }
}
