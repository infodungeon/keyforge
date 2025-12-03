use crate::config::Config;
use crate::core_types::{KeyCode, Layout};
use crate::optimizer::crossover::crossover_uniform;
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
    pub initial_population: Vec<Layout>,
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
            initial_population: Vec::new(),
        }
    }
}

pub struct OptimizationResult {
    pub score: f32,
    pub layout: Layout,
}

pub trait ProgressCallback: Send + Sync {
    fn on_progress(&self, epoch: usize, score: f32, best_layout: &[KeyCode], ips: f32) -> bool;
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
                    // FIXED: Removed the 'false' debug argument here
                    opts.limit_fast,
                    opts.limit_slow,
                    &opts.pinned_keys,
                );

                // Inject initial population if available
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
        let mut gene_pool: Vec<(f32, Layout)> = self.seed_gene_pool();

        let mut rng = if let Some(s) = seed {
            fastrand::Rng::with_seed(s + 9999)
        } else {
            fastrand::Rng::new()
        };

        let mut patience_counter = 0;
        let mut local_best_score = f32::MAX;
        let mut last_print = Instant::now();
        let mut steps_since_last_report = 0;
        let start_time = Instant::now();

        // 3. Main Loop
        for epoch in 0..opts.epochs {
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

            // B. Parallel Tempering
            self.try_tempering(&mut replicas, &mut rng);

            // C. Genetic Crossover (Every 50 epochs)
            if epoch > 0 && epoch % 50 == 0 {
                self.perform_crossover(&mut replicas, &gene_pool, &mut rng);
            }

            // D. Harvest & Update Gene Pool
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

                // Add to Gene Pool if unique and reasonably good
                if r.score < global_best_score * 1.5
                    && !gene_pool.iter().any(|(_, l)| l == &r.layout)
                {
                    gene_pool.push((r.score, r.layout.clone()));
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
            layout: global_best_layout,
        }
    }

    // --- Helpers ---

    fn seed_gene_pool(&self) -> Vec<(f32, Layout)> {
        let mut pool = Vec::new();
        for layout in &self.options.initial_population {
            if layout.len() == self.scorer.key_count {
                let map = mutation::build_pos_map(layout);
                let (s, l, _) = self.scorer.score_full(&map, self.options.limit_slow);
                let imb = if l > 0.0 {
                    (l / self.scorer.geometry.keys.len() as f32 - 0.5).abs() * 200.0
                } else {
                    0.0
                };
                pool.push((s + imb, layout.clone()));
            }
        }
        pool
    }

    fn try_tempering(&self, replicas: &mut [Replica], rng: &mut fastrand::Rng) {
        if replicas.len() < 2 {
            return;
        }

        for i in (0..replicas.len() - 1).rev() {
            let (head, tail) = replicas.split_at_mut(i + 1);
            let r1 = &mut head[i];
            let r2 = &mut tail[0];

            let e1 = r1.score;
            let e2 = r2.score;
            let t1 = r1.temperature;
            let t2 = r2.temperature;

            let delta_beta = (1.0f32 / t1) - (1.0f32 / t2);
            let delta_e = e2 - e1;

            if rng.f32() < (-delta_beta * delta_e).exp() {
                // Swap STATE, but keep TEMPERATURE configuration.
                std::mem::swap(&mut r1.layout, &mut r2.layout);
                std::mem::swap(&mut r1.pos_map, &mut r2.pos_map);
                std::mem::swap(&mut r1.score, &mut r2.score);
                std::mem::swap(&mut r1.left_load, &mut r2.left_load);
                std::mem::swap(&mut r1.total_freq, &mut r2.total_freq);
                std::mem::swap(&mut r1.mutation_weights, &mut r2.mutation_weights);
                std::mem::swap(&mut r1.total_weight, &mut r2.total_weight);
            }
        }
    }

    fn perform_crossover(
        &self,
        replicas: &mut [Replica],
        gene_pool: &[(f32, Layout)],
        rng: &mut fastrand::Rng,
    ) {
        if gene_pool.len() < 2 || replicas.len() <= 1 {
            return;
        }

        let p1_idx = rng.usize(0..gene_pool.len().min(5));
        let p2_idx = rng.usize(0..gene_pool.len());

        let p1 = &gene_pool[p1_idx].1;
        let p2 = &gene_pool[p2_idx].1;

        let key_count = self.scorer.key_count;
        let mut pinned_slots = vec![None; key_count];
        if !self.options.pinned_keys.is_empty() {
            for part in self.options.pinned_keys.split(',') {
                let parts: Vec<&str> = part.split(':').collect();
                if parts.len() == 2 {
                    if let Ok(idx) = parts[0].trim().parse::<usize>() {
                        if idx < key_count {
                            let char_str = parts[1].trim();
                            if let Some(c) = char_str.chars().next() {
                                pinned_slots[idx] = Some(c.to_ascii_lowercase() as KeyCode);
                            }
                        }
                    }
                }
            }
        }

        let child_layout = crossover_uniform(p1, p2, &pinned_slots, rng);

        let target_idx = rng.usize(1..replicas.len());
        replicas[target_idx].inject_layout(&child_layout);
    }
}
