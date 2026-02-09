#![allow(clippy::print_stdout, clippy::print_stderr)]
// apps/keyforge-cli/src/cmd/benchmark.rs

use crate::build_job_config;
use crate::constants::DEFAULT_BENCHMARK_ITERATIONS;
use crate::error::CliError;
use clap::Args;
use keyforge_adapter::loader::AssetLoader;
use keyforge_infra::FsProvider;
use keyforge_model::KeyboardDefinition;

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

    let keycodes_file = args
        .shared
        .keycodes
        .as_deref()
        .unwrap_or(keyforge_model::constants::ASSET_KEYCODES_FILENAME);

    let builder = keyforge_compute::SessionBuilder::new(loader)
        .with_keyboard_def(std::sync::Arc::new(KeyboardDefinition::from_geometry(
            job.to_domain_geometry(),
            "benchmark",
        )))
        .with_corpus(&job.to_domain_corpus_sources())
        .await
        .map_err(|e| CliError::Other(format!("Corpus load failed: {e}")))?
        .with_cost_matrix(&job.to_domain_cost_matrix())
        .await
        .map_err(|e| CliError::Other(format!("Cost matrix load failed: {e}")))?
        .with_keycodes(keycodes_file)
        .await
        .map_err(|e| CliError::Other(format!("Keycodes load failed: {e}")))?
        .with_rubric(keyforge_adapter::conversion::to_domain_rubric(
            &job.to_domain_weights()
                .map_err(|e| CliError::Other(format!("Invalid weights: {e}")))?,
        ));

    let session = builder
        .build()
        .map_err(|e| CliError::Other(format!("Failed to prepare session: {e}")))?;

    // Create a non-zero layout for representative benchmarking
    let mut keycodes = vec![keyforge_model::KeyCode::new(0); session.engine.key_count()];
    for (i, kc) in keycodes.iter_mut().enumerate() {
        if let Ok(i_u16) = u16::try_from(i) {
            *kc = keyforge_model::KeyCode::new(i_u16);
        }
    }
    let benchmark_layout = keyforge_model::Layout::new_unchecked(keycodes);

    let start = std::time::Instant::now();
    for _ in 0..args.iterations {
        let _ = session
            .engine
            .score(&benchmark_layout)
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
        .analyze(&benchmark_layout)
        .map_err(|e| CliError::Other(format!("Analysis Error: {e}")))?;

    // Display standard scoring table
    crate::reports::scoring(&[("Benchmark".to_string(), report.clone())]);

    // Attempt Reality Check comparison
    if let Some(baselines) = crate::reports::load_benchmarks(loader.root()) {
        crate::reports::benchmark_comparison("Benchmark", &report, &baselines);
    } else {
        eprintln!("\n⚠️  Skipping Reality Check: benchmark file not found.");
    }

    Ok(())
}
