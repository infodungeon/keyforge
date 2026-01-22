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
    let mut score_sum = 0.0;
    let default_layout = keyforge_model::Layout::new_unchecked(vec![
        keyforge_model::KeyCode(0);
        session.engine.key_count()
    ]);

    for _ in 0..args.iterations {
        score_sum += session
            .engine
            .score(&default_layout)
            .map_err(|e| CliError::Other(format!("Scoring Error: {e}")))?
            .to_f32();
    }

    let duration = start.elapsed();
    #[allow(clippy::cast_precision_loss)]
    let kops = (args.iterations as f64 / duration.as_secs_f64()) / 1000.0;

    println!(
        "{}",
        serde_json::json!({
            "iterations": args.iterations,
            "duration_ms": duration.as_millis(),
            "kops": kops,
            "checksum": score_sum
        })
    );
    Ok(())
}
