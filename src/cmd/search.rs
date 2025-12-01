use crate::reports;
use clap::Args;
use keyforge::config::Config;
use keyforge::optimizer::{OptimizationOptions, Optimizer, ProgressCallback};
use keyforge::scorer::Scorer;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

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

// Simple callback for CLI logging
struct CliLogger;
impl ProgressCallback for CliLogger {
    fn on_progress(&self, epoch: usize, score: f32, _layout: &[u8], ips: f32) -> bool {
        info!("Ep {:5} | Global Best: {:.0} | {:.2}M/s", epoch, score, ips);
        true // Continue search
    }
}

pub fn run(args: SearchArgs, scorer: Arc<Scorer>, _debug: bool) {
    let mut options = OptimizationOptions::from(&args.config);

    // Apply CLI-specific overrides
    if let Some(t) = args.time {
        options.max_time = Some(Duration::from_secs(t));
    }

    let optimizer = Optimizer::new(scorer, options);
    let attempts = args.attempts.unwrap_or(1);

    let mut overall_best_score = f32::MAX;
    let mut overall_best_layout = Vec::new();

    for i in 1..=attempts {
        info!("➡️  Attempt #{} of {}", i, attempts);

        let seed = args.seed.map(|s| s + (i as u64 * 100));
        let result = optimizer.run(seed, CliLogger);

        if result.score < overall_best_score {
            overall_best_score = result.score;
            overall_best_layout = result.layout_bytes;
        }
    }

    let layout_str = String::from_utf8_lossy(&overall_best_layout).to_string();

    info!("\n=== 🏆 FINAL RESULT ===");
    info!("Score: {:.2}", overall_best_score);
    info!("Layout: {}", layout_str);

    reports::print_layout_grid("OPTIMIZED", &overall_best_layout);
}
