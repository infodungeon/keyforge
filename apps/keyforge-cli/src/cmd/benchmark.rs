#![allow(clippy::print_stdout, clippy::print_stderr)]
use crate::build_job_config;
use crate::constants::DEFAULT_BENCHMARK_ITERATIONS;
use crate::error::CliError;
use clap::Args;
use keyforge_infra::FsProvider;

#[derive(Args, Debug, Clone)]
pub struct BenchmarkArgs {
    #[command(flatten)]
    pub config: crate::cli_args::config::ConfigArgs,
    #[arg(long, default_value_t = DEFAULT_BENCHMARK_ITERATIONS)]
    pub iterations: usize,
    #[command(flatten)]
    pub shared: crate::cmd::shared::SharedArgs,
}

pub async fn run(args: &BenchmarkArgs, loader: &FsProvider) -> Result<(), CliError> {
    let job = build_job_config(loader, &args.shared, args.config.clone())
        .await
        .map_err(|e| CliError::Other(format!("Failed to build job: {e}")))?;

    let builder = keyforge_compute::SessionBuilder::new(loader)
        .with_keyboard_def(std::sync::Arc::new(job.definition.clone()))
        .with_corpus(&job.corpora)
        .await
        .map_err(|e| CliError::Other(format!("Corpus load failed: {e}")))?
        .with_cost_matrix(&job.cost_matrix)
        .await
        .map_err(|e| CliError::Other(format!("Cost matrix load failed: {e}")))?
        .with_keycodes("keycodes.json")
        .await
        .map_err(|e| CliError::Other(format!("Keycodes load failed: {e}")))?
        .with_rubric(keyforge_adapter::conversion::to_domain_rubric(&job.weights));

    let session = builder
        .build()
        .map_err(|e| CliError::Other(format!("Failed to prepare session: {e}")))?;

    let start = std::time::Instant::now();
    let default_layout = keyforge_model::Layout::new_unchecked(vec![
        keyforge_model::KeyCode(0);
        session.engine.key_count()
    ]);

    for _ in 0..args.iterations {
        let _ = session
            .engine
            .score(&default_layout)
            .map_err(|e| CliError::Other(format!("Scoring Error: {e}")))?;
    }

    let duration = start.elapsed();
    #[allow(clippy::cast_precision_loss)]
    let kops = (args.iterations as f64 / duration.as_secs_f64()) / 1000.0;

    println!(
        "\n🚀 Engine Throughput: {:.2} KOPS ({:?} iterations in {:?})",
        kops, args.iterations, duration
    );

    // Perform layout analysis for ergonomic metrics
    let runtime = keyforge_compute::Runtime::from(session);
    let report = runtime
        .analyze(&default_layout)
        .map_err(|e| CliError::Other(format!("Analysis Error: {e}")))?;

    // Display standard scoring table
    crate::reports::scoring(&[("Baseline".to_string(), report.clone())]);

    // Attempt Reality Check comparison
    if let Some(baselines) = crate::reports::load_benchmarks(loader.root()) {
        crate::reports::benchmark_comparison("Current", &report, &baselines);
    } else {
        eprintln!("\n⚠️  Skipping Reality Check: benchmark file not found.");
    }

    Ok(())
}
