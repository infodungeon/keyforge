use crate::reports;
use clap::Args;
use keyforge_core::config::Config;
use keyforge_core::keycodes::KeycodeRegistry;
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

struct CliLogger {
    registry: Arc<KeycodeRegistry>,
}

impl ProgressCallback for CliLogger {
    fn on_progress(&self, epoch: usize, score: f32, layout: &[u16], ips: f32) -> bool {
        // Create a short preview of the layout (first 10 keys)
        // This uses both 'self.registry' and 'layout', resolving the warnings.
        let preview_len = layout.len().min(10);
        let preview: String = layout
            .iter()
            .take(preview_len)
            .map(|&c| self.registry.get_label(c))
            .collect::<Vec<String>>()
            .join("");

        let ellipsis = if layout.len() > 10 { "..." } else { "" };

        info!(
            "Ep {:5} | Best: {:.0} | {:.2}M/s | [ {}{} ]",
            epoch, score, ips, preview, ellipsis
        );
        true
    }
}

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
        let result = optimizer.run(
            seed,
            CliLogger {
                registry: registry.clone(),
            },
        );

        if result.score < overall_best_score {
            overall_best_score = result.score;
            overall_best_layout = result.layout;
        }
    }

    // Convert u16 layout back to readable string for log using Registry
    let layout_str = overall_best_layout
        .iter()
        .map(|&c| registry.get_label(c))
        .collect::<Vec<String>>()
        .join(" ");

    info!("\n=== 🏆 FINAL RESULT ===");
    info!("Score: {:.2}", overall_best_score);
    info!("Layout: {}", layout_str);

    // Pass registry to report printer for nice grid output
    reports::print_layout_grid("OPTIMIZED", &overall_best_layout, &registry);
}
