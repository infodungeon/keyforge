use crate::config::Config;
use crate::optimizer::Replica; // Removed 'mutation'
use crate::scorer::Scorer;
use rayon::prelude::*;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct OptimizationOptions {
    pub num_threads: usize,
    pub temp_min: f32,
    pub temp_max: f32,
    pub epochs: usize,
    pub steps_per_epoch: usize,
    pub patience: usize,
    pub patience_threshold: f32,
    pub limit_fast: usize,
    pub limit_slow: usize,
    pub pinned_keys: String,
    pub max_time: Option<Duration>,
}

impl From<&Config> for OptimizationOptions {
    fn from(cfg: &Config) -> Self {
        Self {
            num_threads: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4),
            temp_min: cfg.search.temp_min,
            temp_max: cfg.search.temp_max,
            epochs: cfg.search.search_epochs,
            steps_per_epoch: cfg.search.search_steps,
            patience: cfg.search.search_patience,
            patience_threshold: cfg.search.search_patience_threshold,
            limit_fast: cfg.search.opt_limit_fast,
            limit_slow: cfg.search.opt_limit_slow,
            pinned_keys: cfg.search.pinned_keys.clone(),
            max_time: None, // Set manually if needed
        }
    }
}

pub struct OptimizationResult {
    pub score: f32,
    pub layout_bytes: Vec<u8>,
}

/// A trait for receiving updates during optimization.
/// Boolean return value indicates if the search should continue (true) or abort (false).
pub trait ProgressCallback: Send + Sync {
    fn on_progress(&self, epoch: usize, score: f32, best_layout: &[u8], ips: f32) -> bool;
}

pub struct Optimizer {
    scorer: Arc<Scorer>,
    options: OptimizationOptions,
}

impl Optimizer {
    pub fn new(scorer: Arc<Scorer>, options: OptimizationOptions) -> Self {
        Self { scorer, options }
    }

    pub fn run<CB: ProgressCallback>(&self, seed: Option<u64>, callback: CB) -> OptimizationResult {
        let opts = &self.options;

        // 1. Initialize Replicas
        let mut replicas: Vec<Replica> = (0..opts.num_threads)
            .map(|i| {
                let progress = i as f32 / (opts.num_threads - 1) as f32;
                let temp = opts.temp_min * (opts.temp_max / opts.temp_min).powf(progress);
                let replica_seed = seed.map(|s| s + i as u64);

                Replica::new(
                    self.scorer.clone(),
                    temp,
                    replica_seed,
                    false, // debug flag inside replica isn't strictly needed for runner
                    opts.limit_fast,
                    opts.limit_slow,
                    &opts.pinned_keys,
                )
            })
            .collect();

        // 2. Global State
        let mut global_best_score = f32::MAX;
        let mut global_best_layout = Vec::new();
        let mut rng = if let Some(s) = seed {
            fastrand::Rng::with_seed(s + 9999)
        } else {
            fastrand::Rng::new()
        };

        // 3. Initial Reset
        let mut patience_counter = 0;
        let mut local_best_score = f32::MAX;
        let mut last_print = Instant::now();
        let mut steps_since_last_report = 0;
        let start_time = Instant::now();

        // 4. Main Loop
        for epoch in 0..opts.epochs {
            // Check Time Limit
            if let Some(limit) = opts.max_time {
                if start_time.elapsed() >= limit {
                    break;
                }
            }

            // A. Evolve in Parallel
            let steps_this_epoch: usize = replicas
                .par_iter_mut()
                .map(|r| {
                    // Temperature-based step scaling
                    let multiplier = if r.temperature > 50.0 {
                        2.5
                    } else if r.temperature > 5.0 {
                        1.5
                    } else {
                        1.0
                    };
                    let adjusted_steps = (opts.steps_per_epoch as f32 * multiplier) as usize;
                    r.evolve(adjusted_steps);
                    adjusted_steps
                })
                .sum();

            steps_since_last_report += steps_this_epoch;

            // B. Parallel Tempering (Swap Replicas)
            for i in (0..opts.num_threads - 1).rev() {
                let j = i + 1;
                let e1 = replicas[i].score;
                let e2 = replicas[j].score;
                let t1 = replicas[i].temperature;
                let t2 = replicas[j].temperature;

                let delta_beta = (1.0f32 / t1) - (1.0f32 / t2);
                let delta_e = e2 - e1;

                if rng.f32() < (-delta_beta * delta_e).exp() {
                    let temp_layout = replicas[i].layout.clone();
                    let temp_score = replicas[i].score;
                    let temp_pos = replicas[i].pos_map;
                    let temp_load = replicas[i].left_load;

                    replicas[i].layout = replicas[j].layout.clone();
                    replicas[i].score = replicas[j].score;
                    replicas[i].pos_map = replicas[j].pos_map;
                    replicas[i].left_load = replicas[j].left_load;

                    replicas[j].layout = temp_layout;
                    replicas[j].score = temp_score;
                    replicas[j].pos_map = temp_pos;
                    replicas[j].left_load = temp_load;
                }
            }

            // C. Check Improvements
            let mut improved = false;
            for r in &replicas {
                if r.score < local_best_score - opts.patience_threshold {
                    local_best_score = r.score;
                    improved = true;
                }
                if r.score < global_best_score {
                    global_best_score = r.score;
                    global_best_layout = r.layout.clone();
                }
            }

            if improved {
                patience_counter = 0;
            } else {
                patience_counter += 1;
            }

            if patience_counter >= opts.patience {
                break;
            }

            // D. Report Progress
            let now = Instant::now();
            let duration = now.duration_since(last_print).as_secs_f32();

            if duration >= 1.0 {
                // Update every 1 second
                let ips = (steps_since_last_report as f32) / duration / 1_000_000.0;
                let keep_going =
                    callback.on_progress(epoch, global_best_score, &global_best_layout, ips);

                if !keep_going {
                    break;
                }

                last_print = now;
                steps_since_last_report = 0;
            }
        }

        OptimizationResult {
            score: global_best_score,
            layout_bytes: global_best_layout,
        }
    }
}
