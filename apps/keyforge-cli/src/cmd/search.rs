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
    let mut options = keyforge_runner::RunnerOptions::default();
    if let Some(t) = args.time {
        options.timeout_sec = t;
    }

    if let Some(kc) = &args.shared.keycodes {
        options.keycodes_file.clone_from(kc);
    } else {
        options.keycodes_file = keyforge_model::constants::ASSET_KEYCODES_FILENAME.into();
    }

    if let Some(s) = args.seed {
        options.seed = Some(s);
    }
    if args.threads > 0 {
        options.threads = args.threads;
    }

    let job = build_job_config(loader, &args.shared, args.config.clone())
        .await
        .map_err(|e| CliError::Other(format!("Failed to build job: {e}")))?;
    let session = keyforge_runner::OptimizationRunner::prepare_session(loader, &job, &options)
        .await
        .map_err(|e| CliError::Other(format!("Failed to prepare session: {e}")))?;

    let stop_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));

    // Task-clii-rev-003: Setup Progress Bar
    let pb = if options.timeout_sec > 0 {
        ProgressBar::new(options.timeout_sec)
    } else {
        ProgressBar::new_spinner()
    };

    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
        .unwrap_or_else(|_| ProgressStyle::default_bar())
        .progress_chars("#>-"));
    
    if options.timeout_sec == 0 {
        pb.set_message("Optimizing layout (Infinite)...");
    } else {
        pb.set_message("Optimizing layout...");
    }

    let callback = ProgressBarCallback {
        stop_flag: stop_flag.clone(),
        pb: pb.clone(),
        start_time: std::time::Instant::now(),
    };

    let result: keyforge_model::OptimizationResult = keyforge_runner::OptimizationRunner::run(
        session,
        "local-cli".into(),
        stop_flag,
        callback,
        options,
        &job,
    )
    .await
    .map_err(|e| CliError::Other(format!("Optimization Error: {e}")))?;

    pb.finish_with_message("Optimization complete.");
    println!(
        "{}",
        serde_json::to_string_pretty(&result)
            .map_err(|e| CliError::Other(format!("JSON Error: {e}")))?
    );
    Ok(())
}
