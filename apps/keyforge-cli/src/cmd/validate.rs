#![allow(clippy::print_stdout, clippy::print_stderr)]
// apps/keyforge-cli/src/cmd/validate.rs

use crate::build_job_config;
use crate::error::CliError;
use clap::Args;
use keyforge_adapter::loader::AssetLoader;
use keyforge_infra::FsProvider;
use keyforge_model::KeyboardDefinition;

#[derive(Args, Debug, Clone)]
pub struct ValidateArgs {
    #[command(flatten)]
    pub config: crate::cli_args::config::ConfigArgs,

    #[arg(short, long)]
    pub layout: String,

    #[command(flatten)]
    pub shared: crate::cmd::shared::SharedArgs,
}

pub async fn run(args: &ValidateArgs, loader: &FsProvider) -> Result<(), CliError> {
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
            "validate",
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

    // Parse provided layout string
    let layout_parsed = keyforge_adapter::conversion::parse_layout_string(
        &args.layout,
        session.engine.key_count(),
        &session.registry,
    )
    .map_err(|e| CliError::Other(format!("Layout parsing error: {e}")))?;

    // Perform layout analysis for ergonomic metrics
    let runtime = keyforge_compute::Runtime::from(session);
    let report = runtime
        .analyze(&layout_parsed)
        .map_err(|e| CliError::Other(format!("Analysis Error: {e}")))?;

    // Display standard scoring table
    crate::reports::scoring(&[("Validation".to_string(), report.clone())]);

    // Attempt Reality Check comparison
    if let Some(baselines) = crate::reports::load_benchmarks(loader.root()) {
        crate::reports::benchmark_comparison("Validation", &report, &baselines);
    }

    Ok(())
}
