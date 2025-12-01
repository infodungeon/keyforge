use crate::config::Config;
use crate::optimizer::crossover::crossover_uniform; // NEW IMPORT
use crate::optimizer::{mutation, Replica};
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
    pub initial_population: Vec<Vec<u8>>, // NEW FIELD
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
            max_time: None,
            initial_population: Vec::new(), // Default empty
        }
    }
}

pub struct OptimizationResult {
    pub score: f32,
    pub layout_bytes: Vec<u8>,
}

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
                let progress = i as f32 / (opts.num_threads.max(2) - 1) as f32;
                let temp = opts.temp_min * (opts.temp_max / opts.temp_min).powf(progress);
                let replica_seed = seed.map(|s| s + i as u64);

                let mut r = Replica::new(
                    self.scorer.clone(),
                    temp,
                    replica_seed,
                    false,
                    opts.limit_fast,
                    opts.limit_slow,
                    &opts.pinned_keys,
                );

                // Inject initial population if available
                // We distribute population items across replicas round-robin style
                if !opts.initial_population.is_empty() {
                    let layout_idx = i % opts.initial_population.len();
                    let layout = &opts.initial_population[layout_idx];
                    if layout.len() == self.scorer.key_count {
                        r.inject_layout(layout);
                    }
                }

                r
            })
            .collect();

        // 2. Global State
        let mut global_best_score = f32::MAX;
        let mut global_best_layout = Vec::new();

        // Gene Pool for Crossover (Top 10 Layouts)
        let mut gene_pool: Vec<(f32, Vec<u8>)> = Vec::new();

        // Seed the gene pool with initial population
        for layout in &opts.initial_population {
            if layout.len() == self.scorer.key_count {
                // We need to score these to sort them
                let map = mutation::build_pos_map(layout);
                let (s, l, _) = self.scorer.score_full(&map, opts.limit_slow);
                // Simple imbalance calc
                // (Note: This duplicates logic in Replica, simplified here for pool seeding)
                // ideally we'd use a helper, but this is sufficient for seeding
                let imb = if l > 0.0 {
                    (l / self.scorer.geometry.keys.len() as f32 - 0.5).abs() * 200.0
                } else {
                    0.0
                };
                gene_pool.push((s + imb, layout.clone()));
            }
        }

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
                    // Swap logic manually to satisfy borrow checker if needed,
                    // or just use std::mem::swap on fields.
                    // Since Replicas are in a vec, we can just swap the data inside.
                    // However, we must retain the TEMPERATURE of the slot.
                    // So we swap everything EXCEPT temperature/config.

                    let tmp_layout = replicas[i].layout.clone();
                    let tmp_score = replicas[i].score;
                    let tmp_pos = replicas[i].pos_map;
                    let tmp_load = replicas[i].left_load;

                    replicas[i].layout = replicas[j].layout.clone();
                    replicas[i].score = replicas[j].score;
                    replicas[i].pos_map = replicas[j].pos_map;
                    replicas[i].left_load = replicas[j].left_load;

                    replicas[j].layout = tmp_layout;
                    replicas[j].score = tmp_score;
                    replicas[j].pos_map = tmp_pos;
                    replicas[j].left_load = tmp_load;
                }
            }

            // C. GENETIC CROSSOVER STEP (New)
            // Every 50 epochs, breed the best layouts
            if epoch > 0 && epoch % 50 == 0 && gene_pool.len() >= 2 {
                // 1. Pick 2 random parents from the top of the gene pool
                let p1_idx = rng.usize(0..gene_pool.len().min(5)); // Bias towards top 5
                let p2_idx = rng.usize(0..gene_pool.len());

                let p1 = &gene_pool[p1_idx].1;
                let p2 = &gene_pool[p2_idx].1;

                // 2. Create Child
                let child_layout = crossover_uniform(p1, p2, &mut rng);

                // 3. Inject into a random HIGH TEMP replica (index > 0)
                // We want to refine the child, not destroy our best low-temp state
                if opts.num_threads > 1 {
                    let target_idx = rng.usize(1..opts.num_threads);
                    replicas[target_idx].inject_layout(&child_layout);
                }
            }

            // D. Harvest & Update Gene Pool
            let mut improved = false;
            for r in &replicas {
                // Update Local/Global Bests
                if r.score < local_best_score - opts.patience_threshold {
                    local_best_score = r.score;
                    improved = true;
                }
                if r.score < global_best_score {
                    global_best_score = r.score;
                    global_best_layout = r.layout.clone();
                }

                // Add to Gene Pool (Simple maintain top 10)
                // Only add if fairly good (e.g. better than 1.2x global best)
                if r.score < global_best_score * 1.5 {
                    // Check strict uniqueness
                    if !gene_pool.iter().any(|(_, l)| l == &r.layout) {
                        gene_pool.push((r.score, r.layout.clone()));
                    }
                }
            }

            // Trim Gene Pool
            gene_pool.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            if gene_pool.len() > 20 {
                gene_pool.truncate(20);
            }

            if improved {
                patience_counter = 0;
            } else {
                patience_counter += 1;
            }

            if patience_counter >= opts.patience {
                break;
            }

            // E. Report Progress
            let now = Instant::now();
            let duration = now.duration_since(last_print).as_secs_f32();

            if duration >= 1.0 {
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
