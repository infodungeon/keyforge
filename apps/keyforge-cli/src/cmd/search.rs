#![allow(clippy::print_stdout, clippy::print_stderr)]
// apps/keyforge-cli/src/cmd/search.rs

use crate::error::CliError;
use crate::{build_job_config, ProgressBarCallback};
use clap::Args;
use indicatif::{ProgressBar, ProgressStyle};
use keyforge_infra::FsProvider;
use std::sync::Arc;

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

pub async fn run(
    args: &SearchArgs,
    loader: &FsProvider,
    _config: &keyforge_infra::config::CommonConfig,
) -> Result<(), CliError> {
    let job = build_job_config(loader, &args.shared, args.config.clone())
        .await
        .map_err(|e| CliError::Other(format!("Failed to build job: {e}")))?;

    let keycodes_file = args
        .shared
        .keycodes
        .as_deref()
        .unwrap_or(keyforge_model::constants::ASSET_KEYCODES_FILENAME);

    let builder = keyforge_compute::SessionBuilder::new(loader)
        .with_keyboard_def(std::sync::Arc::new(job.definition.clone()))
        .with_corpus(&job.corpora)
        .await
        .map_err(|e| CliError::Other(format!("Corpus load failed: {e}")))?
        .with_cost_matrix(&job.cost_matrix)
        .await
        .map_err(|e| CliError::Other(format!("Cost matrix load failed: {e}")))?
        .with_keycodes(keycodes_file)
        .await
        .map_err(|e| CliError::Other(format!("Keycodes load failed: {e}")))?
        .with_rubric(keyforge_adapter::conversion::to_domain_rubric(&job.weights))
        .with_config(keyforge_model::SearchConfig::Annealing {
            steps: job.params.get_search_steps(),
            start_temp: job.params.get_temp_max(),
            end_temp: job.params.get_temp_min(),
            seed: args.seed.unwrap_or(job.params.seed.unwrap_or(42)),
            patience: job.params.get_search_patience(),
            reheats: job.params.get_reheats(),
            reheat_factor: job.params.get_reheat_factor(),
            include_thumbs: job.params.include_thumbs,
        });

    let session = builder
        .build()
        .map_err(|e| CliError::Other(format!("Failed to prepare session: {e}")))?;

    let stop_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let timeout_sec = args.time.unwrap_or(0);

    // Setup Progress Bar
    let pb = if timeout_sec > 0 {
        ProgressBar::new(timeout_sec)
    } else {
        ProgressBar::new_spinner()
    };

    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_bar())
            .progress_chars("#>-"),
    );

    if timeout_sec == 0 {
        pb.set_message("Optimizing layout (Infinite)...");
    } else {
        pb.set_message("Optimizing layout...");
    }

    let callback = ProgressBarCallback {
        stop_flag: stop_flag.clone(),
        pb: pb.clone(),
        start_time: std::time::Instant::now(),
    };

    let runtime = keyforge_compute::Runtime::from(session);
    let result: keyforge_model::OptimizationResult = runtime
        .run_optimization(callback, &job.pinned_keys)
        .await
        .map_err(|e| CliError::Other(format!("Optimization Error: {e}")))?;

    pb.finish_with_message("Optimization complete.");

    // Perform final analysis for reporting
    let report = runtime
        .analyze(&result.layout)
        .map_err(|e| CliError::Other(format!("Analysis Error: {e}")))?;

    // Display standard scoring table
    crate::reports::scoring(&[("Optimized".to_string(), report.clone())]);

    // Attempt Reality Check comparison
    if let Some(baselines) = crate::reports::load_benchmarks(loader.root()) {
        crate::reports::benchmark_comparison("Optimized", &report, &baselines);
    }

    Ok(())
}
