use crate::reports;
use clap::Args;
use keyforge::config::Config;
use keyforge::optimizer::{mutation, Replica};
use keyforge::scorer::Scorer;
use rayon::prelude::*;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Args, Debug, Clone)]
pub struct SearchArgs {
    #[command(flatten)]
    pub config: Config,

    #[arg(short = 'T', long)]
    pub time: Option<u64>,

    #[arg(short = 'a', long)]
    pub attempts: Option<usize>,

    #[arg(short = 'S', long)]
    pub seed: Option<u64>,
}

pub fn run(args: SearchArgs, scorer: Arc<Scorer>, debug: bool) {
    let num_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    println!(
        "🔥 Spawning {} Replicas for Parallel Tempering",
        num_threads
    );

    let min_temp = args.config.search.temp_min;
    let max_temp = args.config.search.temp_max;

    let mut replicas: Vec<Replica> = (0..num_threads)
        .map(|i| {
            let progress = i as f32 / (num_threads - 1) as f32;
            let temp = min_temp * (max_temp / min_temp).powf(progress);
            let replica_seed = args.seed.map(|s| s + i as u64);

            Replica::new(
                scorer.clone(),
                temp,
                replica_seed,
                debug,
                args.config.search.opt_limit_fast,
                args.config.search.opt_limit_slow,
            )
        })
        .collect();

    let mut rng = if let Some(s) = args.seed {
        fastrand::Rng::with_seed(s + 9999)
    } else {
        fastrand::Rng::new()
    };

    let max_duration = args.time.map(Duration::from_secs);
    let max_attempts = match (args.time, args.attempts) {
        (_, Some(a)) => a,
        (Some(_), None) => usize::MAX,
        (None, None) => 1,
    };

    let total_start_time = Instant::now();
    let mut global_best_score = f32::MAX;
    let mut global_best_layout = String::new();
    let mut attempt_count = 0;

    let target_epochs = args.config.search.search_epochs;
    let target_steps = args.config.search.search_steps;
    let target_patience = args.config.search.search_patience;

    while attempt_count < max_attempts {
        attempt_count += 1;
        if let Some(limit) = max_duration {
            if total_start_time.elapsed() >= limit {
                break;
            }
        }

        println!(
            "\n➡️  Attempt #{} (Best: {:.0})",
            attempt_count, global_best_score
        );

        for (i, r) in replicas.iter_mut().enumerate() {
            if let Some(s) = args.seed {
                r.rng = fastrand::Rng::with_seed(s + (attempt_count as u64 * 100) + i as u64);
            } else {
                r.rng = fastrand::Rng::new();
            }

            let mut layout;
            let mut pos_map;
            let key_count = scorer.key_count;
            loop {
                layout = mutation::generate_tiered_layout(
                    &mut r.rng,
                    &scorer.defs,
                    &scorer.geometry,
                    key_count,
                );
                pos_map = mutation::build_pos_map(&layout);
                let critical = scorer.defs.get_critical_bigrams();
                if !mutation::fails_sanity(&pos_map, &critical, &scorer.geometry) {
                    break;
                }
            }

            let (base, left, total) =
                scorer.score_full(&pos_map, args.config.search.opt_limit_fast);

            let mut score = base;
            if total > 0.0 {
                let ratio = left / total;
                let dist = (ratio - 0.5).abs();
                if dist > scorer.weights.allowed_hand_balance_deviation() {
                    score += dist * scorer.weights.penalty_imbalance;
                }
            }

            r.layout = layout;
            r.pos_map = pos_map;
            r.score = score;
            r.left_load = left;
            r.total_freq = total;
            r.current_limit = args.config.search.opt_limit_fast;
        }

        let mut patience_counter = 0;
        let mut local_best_score = f32::MAX;
        let mut last_print = Instant::now();
        let mut steps_since_last_report = 0;

        for epoch in 0..target_epochs {
            if let Some(limit) = max_duration {
                if total_start_time.elapsed() >= limit {
                    break;
                }
            }

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
                    let adjusted_steps = (target_steps as f32 * multiplier) as usize;
                    r.evolve(adjusted_steps);
                    adjusted_steps
                })
                .sum();

            steps_since_last_report += steps_this_epoch;

            for i in (0..num_threads - 1).rev() {
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

            let mut improved = false;
            for r in &replicas {
                if r.score < local_best_score - args.config.search.search_patience_threshold {
                    local_best_score = r.score;
                    improved = true;
                }
                if r.score < global_best_score {
                    global_best_score = r.score;
                    global_best_layout = String::from_utf8_lossy(&r.layout).to_string();
                }
            }

            if improved {
                patience_counter = 0;
            } else {
                patience_counter += 1;
            }

            if patience_counter >= target_patience {
                println!("    Converged at {:.2}. Restarting...", local_best_score);
                break;
            }

            let report_interval = if debug { 10 } else { 50 };
            if epoch % report_interval == 0 && epoch > 0 {
                let now = Instant::now();
                let duration = now.duration_since(last_print).as_secs_f32();
                let ips = (steps_since_last_report as f32) / duration / 1_000_000.0;
                println!(
                    "Ep {:5} | Local: {:.0} | Global: {:.0} | {:.2}M/s",
                    epoch, local_best_score, global_best_score, ips
                );
                last_print = now;
                steps_since_last_report = 0;
            }
        }
    }

    println!("\n=== 🏆 FINAL RESULT ===");
    println!("Score: {:.2}", global_best_score);
    println!("Layout: {}", global_best_layout);
    // Use crate::reports for the helper function
    reports::print_layout_grid("OPTIMIZED", global_best_layout.as_bytes());
}
