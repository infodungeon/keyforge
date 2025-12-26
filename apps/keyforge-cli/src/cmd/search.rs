use clap::Args;
use indicatif::{ProgressBar, ProgressStyle};
use keyforge_compute::Runtime;
use keyforge_core::ProgressCallback;

#[derive(Args, Debug, Clone)]
pub struct SearchArgs {
    #[command(flatten)]
    pub config: crate::cli_args::config::ConfigArgs,

    #[arg(short = 'T', long)]
    pub time: Option<u64>,

    #[arg(short = 'a', long)]
    pub attempts: Option<usize>,

    #[arg(short = 'S', long)]
    pub seed: Option<u64>,

    #[arg(long, default_value_t = 0)]
    pub threads: usize,

    #[command(flatten)]
    pub shared: crate::cmd::shared::SharedArgs,
}

struct CliProgress {
    bar: ProgressBar,
}

impl ProgressCallback for CliProgress {
    fn on_progress(&self, step: usize, score: f32, _layout: &[u16], ips: f32) -> bool {
        self.bar.set_position(step as u64);
        self.bar
            .set_message(format!("Score: {:.0} | {:.2} M/s", score, ips));

        // Check global interrupt
        if crate::is_interrupted() {
            return false;
        }
        true
    }
}

pub fn run(args: SearchArgs, mut runtime: Runtime) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize global thread pool if requested
    if args.threads > 0 {
        let _ = rayon::ThreadPoolBuilder::new()
            .num_threads(args.threads)
            .build_global();
    }

    eprintln!("🔎 Starting optimisation…");

    // Apply overrides to the runtime's local config copy
    if let Some(s) = args.seed {
        // FIX: Irrefutable pattern - used direct destructuring
        let keyforge_model::SearchConfig::Annealing { ref mut seed, .. } = runtime.search_config;
        *seed = s;
    }

    // FIX: Irrefutable pattern - used direct destructuring
    let keyforge_model::SearchConfig::Annealing { steps, .. } = runtime.search_config;

    let pb = ProgressBar::new(steps as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
        .unwrap()
        .progress_chars("#>-"));

    let callback = CliProgress { bar: pb.clone() };

    let result = runtime.optimize(callback);

    pb.finish_with_message("Done");

    eprintln!("\n=== �� FINAL RESULT ===");
    eprintln!("Score: {:.3}", result.score);

    let layout_str = result
        .layout
        .keys
        .iter()
        .map(|&c| runtime.registry.get_label(c))
        .collect::<Vec<String>>()
        .join(" ");

    println!("{}", layout_str);
    Ok(())
}
