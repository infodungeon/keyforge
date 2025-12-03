use crate::reports;
use clap::Args;
use keyforge_core::config::Config;
use keyforge_core::keycodes::KeycodeRegistry; // NEW
use keyforge_core::optimizer::{OptimizationOptions, Optimizer, ProgressCallback};
use keyforge_core::scorer::Scorer;
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

struct CliLogger;
impl ProgressCallback for CliLogger {
    // CHANGED: &[u8] -> &[u16]
    fn on_progress(&self, epoch: usize, score: f32, _layout: &[u16], ips: f32) -> bool {
        info!("Ep {:5} | Global Best: {:.0} | {:.2}M/s", epoch, score, ips);
        true
    }
}

// CHANGED: Added registry argument
pub fn run(args: SearchArgs, scorer: Arc<Scorer>, registry: Arc<KeycodeRegistry>, _debug: bool) {
    let mut options = OptimizationOptions::from(&args.config);

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
            overall_best_layout = result.layout; // result.layout is Vec<u16>
        }
    }

    // Convert u16 layout back to readable string for log
    let layout_str = overall_best_layout
        .iter()
        .map(|&c| registry.get_label(c))
        .collect::<Vec<String>>()
        .join(" ");

    info!("\n=== 🏆 FINAL RESULT ===");
    info!("Score: {:.2}", overall_best_score);
    info!("Layout: {}", layout_str);

    // Pass registry to report printer
    reports::print_layout_grid("OPTIMIZED", &overall_best_layout, &registry);
}
