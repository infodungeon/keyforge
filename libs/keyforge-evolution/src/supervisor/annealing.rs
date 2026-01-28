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
use keyforge_model::constants::{
    DEFAULT_REPORT_DIVISOR, MIN_REPORT_INTERVAL, SCORE_SCALE, TEMP_UNDERFLOW_THRESHOLD,
};
use keyforge_model::types::{
    IterationCount, PatienceCount, ReheatCount, ScalingFactor, Seed, Temperature,
};
use keyforge_model::{KeyCode, Layout};
use keyforge_physics::ScoringEngine;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

struct ProgressReporter {
    tx: mpsc::SyncSender<(IterationCount, f32, Vec<KeyCode>, f32)>,
    report_interval: IterationCount,
    last_report_time: Instant,
    last_report_step: IterationCount,
}

impl ProgressReporter {
    fn new(
        tx: mpsc::SyncSender<(IterationCount, f32, Vec<KeyCode>, f32)>,
        total_steps: IterationCount,
        start_time: Instant,
    ) -> Self {
        let report_interval =
            IterationCount((total_steps.0 / DEFAULT_REPORT_DIVISOR).max(MIN_REPORT_INTERVAL));

        Self {
            tx,
            report_interval,
            last_report_time: start_time,
            last_report_step: IterationCount(0),
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn report(&mut self, step: IterationCount, state: &SearchState, time_keeper: &impl TimeKeeper) {
        if step.0.is_multiple_of(self.report_interval.0) {
            let now = time_keeper.now();
            let elapsed = time_keeper.elapsed(self.last_report_time).as_secs_f32();
            let steps_done = if step.0 == 0 {
                0
            } else {
                step.0 - self.last_report_step.0
            };

            let ips = if elapsed > 0.0 {
                steps_done as f32 / elapsed
            } else {
                0.0
            };

            let score_f32 = state.best_score as f32 / SCORE_SCALE;
            let layout_snapshot = state.best_layout().keys().to_vec();

            let _ = self.tx.try_send((step, score_f32, layout_snapshot, ips));

            if step.0 > 0 {
                self.last_report_time = now;
                self.last_report_step = step;
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AnnealingConfig {
    pub steps: IterationCount,
    pub start_temp: Temperature,
    pub end_temp: Temperature,
    pub seed: Seed,
    pub patience: PatienceCount,
    pub reheats: ReheatCount,
    pub reheat_factor: ScalingFactor,
}

impl AnnealingConfig {
    /// Creates a new `AnnealingConfig` with the specified search parameters.
    ///
    /// # Errors
    /// Returns `EvolutionError::Config` if any of the parameters are invalid.
    pub fn new(
        steps: IterationCount,
        start_temp: Temperature,
        end_temp: Temperature,
        seed: Seed,
        patience: PatienceCount,
        reheats: ReheatCount,
        reheat_factor: ScalingFactor,
    ) -> Result<Self, EvolutionError> {
        if steps.0 == 0 {
            return Err(EvolutionError::Config(
                "Steps must be greater than 0".into(),
            ));
        }
        if reheats.0 > 0 && start_temp.0 <= f32::EPSILON {
            return Err(EvolutionError::Config(
                "Reheats require a positive start temperature".into(),
            ));
        }
        if start_temp.0 < 0.0 || end_temp.0 < 0.0 {
            return Err(EvolutionError::Config(
                "Temperatures must be non-negative".into(),
            ));
        }
        if reheat_factor.0 <= 0.0 {
            return Err(EvolutionError::Config("Reheat factor must be > 0.0".into()));
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

#[derive(Debug)]
pub struct Optimizer<'a, M: MutationOperator, A: AcceptanceCriteria, T: TimeKeeper> {
    engine: &'a dyn ScoringEngine,
    config: AnnealingConfig,
    rng: Xoshiro256PlusPlus,
    mutation: M,
    acceptance: A,
    time_keeper: T,
}

impl<'a, M: MutationOperator, A: AcceptanceCriteria, T: TimeKeeper> Optimizer<'a, M, A, T> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        engine: &'a dyn ScoringEngine,
        config: AnnealingConfig,
        mutation: M,
        acceptance: A,
        time_keeper: T,
    ) -> Self {
        let rng = if config.seed.0 == 0 {
            Xoshiro256PlusPlus::from_os_rng()
        } else {
            Xoshiro256PlusPlus::seed_from_u64(config.seed.0)
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

    /// Executes the optimization loop until completion or abortion.
    ///
    /// # Errors
    /// Returns `EvolutionError::Aborted` if the process is cancelled via the callback.
    pub fn run<CB: ProgressCallback>(
        &mut self,
        initial_layout: Option<Layout>,
        callback: CB,
    ) -> Result<Layout, EvolutionError> {
        let mut state = self.initialize_state(initial_layout)?;
        let cooling_rate = self.calculate_cooling_rate();
        let start_time = self.time_keeper.now();
        let abort_flag = Arc::new(std::sync::atomic::AtomicU8::new(0)); // 0=Run, 1=Stop, 2=Abort
        let (tx, rx) = mpsc::sync_channel::<(IterationCount, f32, Vec<KeyCode>, f32)>(1);
        let status_ref = abort_flag.clone();

        let mut reporter = ProgressReporter::new(tx, self.config.steps, start_time);

        thread::scope(|s| {
            s.spawn(move || {
                while let Ok((step, score, layout, ips)) = rx.recv() {
                    match callback.on_progress(step.0, score, &layout, ips) {
                        crate::OptimizationControl::Continue => {}
                        crate::OptimizationControl::Stop => {
                            status_ref.store(1, Ordering::Relaxed);
                            break;
                        }
                        crate::OptimizationControl::Abort => {
                            status_ref.store(2, Ordering::Relaxed);
                            break;
                        }
                    }
                }
            });

            let mut steps_since_improvement = 0;
            let mut reheats_left = self.config.reheats.0;
            let mut result = Ok(());

            for step in 0..self.config.steps.0 {
                if step % 1000 == 0 {
                    let status = abort_flag.load(Ordering::Relaxed);
                    if status == 1 {
                        break;
                    } else if status == 2 {
                        result = Err(EvolutionError::Aborted);
                        break;
                    }
                }

                let improved = self.step(&mut state)?;
                if improved {
                    steps_since_improvement = 0;
                } else {
                    steps_since_improvement += 1;
                }

                if steps_since_improvement > self.config.patience.0 && reheats_left > 0 {
                    state.reheat_from_best(self.config.start_temp, self.config.reheat_factor);
                    reheats_left -= 1;
                    steps_since_improvement = 0;
                }

                Self::update_temperature(&mut state, cooling_rate);
                reporter.report(IterationCount(step), &state, &self.time_keeper);
            }

            drop(reporter);
            result?;

            if abort_flag.load(Ordering::Relaxed) != 0 {
                return Err(EvolutionError::Aborted);
            }

            Ok(state.best_layout().clone())
        })
    }

    fn initialize_state(
        &mut self,
        initial_layout: Option<Layout>,
    ) -> Result<SearchState, EvolutionError> {
        let layout = initial_layout.unwrap_or_else(|| {
            #[allow(clippy::cast_possible_truncation)]
            let keys: Vec<KeyCode> = (0..self.engine.key_count())
                .map(|i| KeyCode(i as u16))
                .collect();
            Layout::new_unchecked(keys)
        });

        let initial_score = self.engine.score(&layout)?.0;
        SearchState::new(layout, initial_score, self.config.start_temp)
    }

    fn calculate_cooling_rate(&self) -> f32 {
        if self.config.steps.0 > 0 && self.config.start_temp.0 > f32::EPSILON {
            #[allow(clippy::cast_precision_loss)]
            let steps_f32 = self.config.steps.0 as f32;
            (self.config.end_temp.0 / self.config.start_temp.0).powf(1.0 / steps_f32)
        } else {
            0.0
        }
    }

    fn step(&mut self, state: &mut SearchState) -> Result<bool, EvolutionError> {
        if let Some(proposal) = self.mutation.propose(
            self.engine,
            state.layout(),
            state.pos_map(),
            &mut self.rng,
            state.temperature.0,
        )? {
            if self
                .acceptance
                .should_accept(proposal.delta, state.temperature.0, &mut self.rng)
            {
                state.apply_mutation(proposal.action)?;
                state.current_score = state.current_score.checked_add(proposal.delta).unwrap_or(
                    if proposal.delta > 0 {
                        i64::MAX
                    } else {
                        i64::MIN
                    },
                );

                if state.current_score < state.best_score {
                    state.update_best();
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn update_temperature(state: &mut SearchState, cooling_rate: f32) {
        state.temperature *= cooling_rate;
        if state.temperature.0 < TEMP_UNDERFLOW_THRESHOLD {
            state.temperature = Temperature(0.0);
        }
    }
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;
    use crate::errors::EvolutionError;
    use crate::supervisor::strategies::{CoolingAnnealing, GroupMutation};
    use crate::supervisor::traits::{
        MutationAction, MutationOperator, MutationProposal, RealTimeKeeper, TimeKeeper,
    };
    use crate::supervisor::AnnealingConfig;
    use crate::{OptimizationControl, ProgressCallback};
    use keyforge_model::types::{ColIndex, FingerIndex, HandIndex, KeyCode, KeyIndex, RowIndex};
    use keyforge_model::{Corpus, CostModel, KeyNode, Keyboard, Layout, Rubric};
    use keyforge_physics::{EngineCompilationContext, EngineFactory, ScoringEngine};
    use rand::Rng;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    #[derive(Clone)]
    struct ReportingCallback(Arc<AtomicUsize>);
    impl ProgressCallback for ReportingCallback {
        fn on_progress(
            &self,
            _epoch: usize,
            _score: f32,
            _layout: &[KeyCode],
            _ips: f32,
        ) -> OptimizationControl {
            self.0.fetch_add(1, Ordering::SeqCst);
            OptimizationControl::Continue
        }
    }

    #[derive(Clone, Copy)]
    struct BreakCallback;
    impl ProgressCallback for BreakCallback {
        fn on_progress(
            &self,
            _epoch: usize,
            _score: f32,
            _layout: &[KeyCode],
            _ips: f32,
        ) -> OptimizationControl {
            OptimizationControl::Abort
        }
    }

    #[derive(Debug, Clone, Copy)]
    struct SaturatingMutation;
    impl MutationOperator for SaturatingMutation {
        fn propose(
            &self,
            _engine: &dyn ScoringEngine,
            _layout: &Layout,
            _pos_map: &[keyforge_model::KeyIndex],
            _rng: &mut impl Rng,
            _temp: f32,
        ) -> Result<Option<MutationProposal>, EvolutionError> {
            Ok(Some(MutationProposal {
                action: MutationAction::Swap(KeyIndex(0), KeyIndex(1)),
                delta: 0,
            }))
        }
    }

    #[derive(Clone, Copy)]
    struct MockTime;
    impl TimeKeeper for MockTime {
        fn now(&self) -> Instant {
            Instant::now()
        }
        fn elapsed(&self, _since: Instant) -> Duration {
            Duration::from_millis(1)
        }
    }

    #[derive(Clone, Copy)]
    struct ZeroTime;
    impl TimeKeeper for ZeroTime {
        fn now(&self) -> Instant {
            Instant::now()
        }
        fn elapsed(&self, _since: Instant) -> Duration {
            Duration::ZERO
        }
    }

    #[derive(Debug, Clone, Copy)]
    struct StagnantMutation;
    impl MutationOperator for StagnantMutation {
        fn propose(
            &self,
            _engine: &dyn ScoringEngine,
            _layout: &Layout,
            _pos_map: &[keyforge_model::KeyIndex],
            _rng: &mut impl Rng,
            _temp: f32,
        ) -> Result<Option<MutationProposal>, EvolutionError> {
            Ok(None)
        }
    }

    fn setup_test_engine(size: usize) -> Box<dyn ScoringEngine> {
        let keys: Vec<_> = (0..size)
            .map(|i| KeyNode {
                index: i,
                label: format!("k{i}"),
                hand: HandIndex(u8::try_from(i % 2).unwrap_or(0)),
                finger: FingerIndex::new_unchecked(u8::try_from(i % 5).unwrap_or(0)),
                row: RowIndex(i8::try_from(i / 10).unwrap_or(0)),
                col: ColIndex(i8::try_from(i % 10).unwrap_or(0)),
                x: (i % 10) as f32,
                y: (i / 10) as f32,
                is_home: false,
                ..Default::default()
            })
            .collect();
        let kb = {
            use keyforge_model::types::RowIndex;
            Arc::new(
                Keyboard::new(keys, RowIndex(1), "test".into()).expect("Failed to create keyboard"),
            )
        };
        let mut corpus_val = Corpus::default();
        let mut bigrams = Vec::new();
        for i in 0..size.saturating_sub(1) {
            bigrams.push((i as u16, (i + 1) as u16, 100));
        }
        corpus_val.bigrams = bigrams.into();
        let corpus = Arc::new(corpus_val);
        let cost_json = r#"{"version": "2.0", "description": "Test", "unit": "pts", "models": {"model_a_row_staggered": {"description": "Test", "static_costs": {"universal_hand": {"thumb": {"pos_1": 1.0}, "index": {"base": {"0": 1.0}}, "middle": {"base": {"0": 1.0}}, "ring": {"base": {"0": 1.0}}, "pinky": {"base": {"0": 1.0}}}}}}, "dynamic_rules": {"sequence_modifiers": {}, "penalties": {}, "constraints": {}}}"#;
        let cost_model: Arc<CostModel> =
            Arc::new(serde_json::from_str(cost_json).expect("Failed to parse cost model"));
        let rubric = Arc::new(Rubric::default());

        EngineFactory::new_generic(&keyforge_physics::ScoringContext::new(
            kb,
            corpus,
            rubric,
            cost_model,
            keyforge_model::config::EngineConfig::default(),
        ))
        .expect("Failed to compile engine")
    }

    #[test]
    fn test_optimizer_basic_loop() {
        let engine = setup_test_engine(10);
        let mutation = GroupMutation {
            unlocked_indices: vec![0, 1, 2],
            start_temp: 1.0,
            end_temp: 0.1,
        };
        let acceptance = CoolingAnnealing;
        let config = AnnealingConfig::new(
            IterationCount(100),
            Temperature(100.0),
            Temperature(0.1),
            Seed(42),
            PatienceCount(5),
            ReheatCount(2),
            ScalingFactor(2.0),
        )
        .unwrap();
        let mut optimizer = Optimizer::new(
            engine.as_ref(),
            config,
            mutation,
            acceptance,
            RealTimeKeeper,
        );
        optimizer.run(None, crate::NoOpCallback).unwrap();
    }

    #[test]
    fn test_ips_underflow() {
        let (tx, _rx) = std::sync::mpsc::sync_channel(1);
        let mut reporter = ProgressReporter::new(tx, IterationCount(100), Instant::now());
        let keys = vec![KeyCode(0); 10];
        let layout = Layout::new_unchecked(keys);
        let state = SearchState::new(layout, 0, Temperature(1.0)).unwrap();
        reporter.report(IterationCount(0), &state, &ZeroTime);
    }
}
