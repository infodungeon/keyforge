use super::state::SearchState;
use super::traits::{AcceptanceCriteria, MutationOperator};
use crate::ProgressCallback;
use keyforge_model::Layout;
use keyforge_physics::ScoringEngine;
use keyforge_protocol::constants::SCORE_SCALE;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;
use std::time::Instant;

pub struct Optimizer<'a, M: MutationOperator, A: AcceptanceCriteria> {
    engine: &'a ScoringEngine,
    total_steps: usize,
    start_temp: f32,
    end_temp: f32,
    rng: Xoshiro256PlusPlus,
    mutation: M,
    acceptance: A,
    patience: usize,
    reheats: usize,
    reheat_factor: f32,
}

impl<'a, M: MutationOperator, A: AcceptanceCriteria> Optimizer<'a, M, A> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        engine: &'a ScoringEngine,
        steps: usize,
        start_temp: f32,
        end_temp: f32,
        seed: u64,
        mutation: M,
        acceptance: A,
        patience: usize,
        reheats: usize,
        reheat_factor: f32,
    ) -> Self {
        let rng = if seed == 0 {
            Xoshiro256PlusPlus::from_entropy()
        } else {
            Xoshiro256PlusPlus::seed_from_u64(seed)
        };

        Self {
            engine,
            total_steps: steps,
            start_temp,
            end_temp,
            rng,
            mutation,
            acceptance,
            patience,
            reheats,
            reheat_factor,
        }
    }

    pub fn run<CB: ProgressCallback>(
        &mut self,
        initial_layout: Option<Layout>,
        callback: CB,
    ) -> Layout {
        let layout = initial_layout.unwrap_or_else(|| {
            let keys: Vec<u16> = (0..self.engine.key_count()).map(|i| i as u16).collect();
            Layout::new(keys)
        });

        let initial_score = self.engine.score_raw(&layout.keys);
        let mut state = SearchState::new(layout, initial_score, self.start_temp);

        let cooling_rate = if self.total_steps > 0 {
            (self.end_temp / self.start_temp).powf(1.0 / self.total_steps as f32)
        } else {
            0.0
        };

        let report_interval = (self.total_steps / 100).max(1000);
        let start_time = Instant::now();
        let mut last_report_time = start_time;
        let mut last_report_step = 0;

        let mut steps_since_improvement = 0;
        let mut reheats_left = self.reheats;

        for step in 0..self.total_steps {
            if let Some(proposal) = self.mutation.propose(
                self.engine,
                &state.current_layout,
                &state.pos_map,
                &mut self.rng,
            ) {
                if self
                    .acceptance
                    .should_accept(proposal.delta, state.temperature, &mut self.rng)
                {
                    proposal
                        .action
                        .apply(&mut state.current_layout, &mut state.pos_map);

                    state.current_score = state
                        .current_score
                        .checked_add(proposal.delta)
                        .unwrap_or(if proposal.delta > 0 {
                            i64::MAX
                        } else {
                            i64::MIN
                        });

                    if state.current_score < state.best_score {
                        state.best_score = state.current_score;
                        state.best_layout = state.current_layout.clone();
                        steps_since_improvement = 0;
                    } else {
                        steps_since_improvement += 1;
                    }
                } else {
                    steps_since_improvement += 1;
                }
            }

            // Reheating Logic
            if steps_since_improvement > self.patience && reheats_left > 0 {
                state.temperature = self.start_temp * self.reheat_factor; // Boost temp
                state.current_layout = state.best_layout.clone(); // Reset to best
                state.current_score = state.best_score;
                // Rebuild pos_map for best layout
                state.pos_map.fill(255);
                for (i, &code) in state.current_layout.keys.iter().enumerate() {
                    if (code as usize) < state.pos_map.len() {
                        state.pos_map[code as usize] = i as u16;
                    }
                }

                reheats_left -= 1;
                steps_since_improvement = 0;
            }

            state.temperature *= cooling_rate;
            if state.temperature < 1e-10 {
                state.temperature = 0.0;
            }

            if step > 0 && step % report_interval == 0 {
                let now = Instant::now();
                let elapsed = now.duration_since(last_report_time).as_secs_f32();
                let steps_done = step - last_report_step;

                let ips = if elapsed > 0.0 {
                    (steps_done as f32 / elapsed) / 1_000_000.0
                } else {
                    0.0
                };

                let score_f32 = state.best_score as f32 / SCORE_SCALE;
                if !callback.on_progress(step, score_f32, &state.best_layout.keys, ips) {
                    break;
                }

                last_report_time = now;
                last_report_step = step;
            }
        }

        state.best_layout
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::supervisor::traits::{MutationOperator, MutationProposal, MutationAction};
    use keyforge_model::{Keyboard, KeyNode, Corpus, Rubric};

    fn setup_test_engine() -> ScoringEngine {
        let keys: Vec<_> = (0..2).map(|i| KeyNode {
            id: i, label: format!("k{}", i), hand: (i % 2) as u8, finger: (i % 5) as u8,
            row: (i / 10) as i8, col: (i % 10) as i8, x: (i % 10) as f32, y: (i / 10) as f32, is_home: false,
        }).collect();
        let kb = Keyboard::new(keys, 1);
        let mut corpus = Corpus::default();
        corpus.bigrams.push((0, 1, 100)); // Non-zero score
        ScoringEngine::new(&kb, &corpus, &Rubric::default(), &[])
    }

    struct SaturatingMutation;
    impl MutationOperator for SaturatingMutation {
        fn propose(&self, _engine: &ScoringEngine, _layout: &Layout, _pos_map: &[u16], _rng: &mut impl rand::Rng) -> Option<MutationProposal> {
            Some(MutationProposal {
                delta: i64::MAX, // Force overflow
                action: MutationAction::Swap(0, 1),
            })
        }
    }

    #[test]
    fn test_saturation_coverage() {
        let engine = setup_test_engine();
        let mut opt = Optimizer::new(&engine, 2, 1.0, 0.1, 42, SaturatingMutation, crate::supervisor::strategies::CoolingAnnealing, 10, 0, 1.0);
        opt.run(None, crate::NoOpCallback);
    }

    struct BreakCallback;
    impl ProgressCallback for BreakCallback {
        fn on_progress(&self, _epoch: usize, _score: f32, _layout: &[u16], _ips: f32) -> bool {
            false
        }
    }

    #[test]
    fn test_callback_break_coverage() {
        let engine = setup_test_engine();
        // Set steps > 1000 to hit report_interval
        let mut opt = Optimizer::new(&engine, 1001, 1.0, 0.1, 42, SaturatingMutation, crate::supervisor::strategies::CoolingAnnealing, 10, 0, 1.0);
        opt.run(None, BreakCallback);
    }
}
