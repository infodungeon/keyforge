#![allow(clippy::print_stdout, clippy::print_stderr)]
// apps/keyforge-cli/src/cmd/validate.rs

use crate::build_job_config;
use crate::error::CliError;
use clap::Args;
use keyforge_infra::FsProvider;
use std::path::Path;

#[derive(Args, Debug, Clone)]
pub struct ValidateArgs {
    #[arg(short, long)]
    pub layout: Option<String>,

    #[command(flatten)]
    pub config: crate::cli_args::config::ConfigArgs,

    #[command(flatten)]
    pub shared: crate::cmd::shared::SharedArgs,
}

pub async fn run(args: &ValidateArgs, loader: &FsProvider, _root: &Path) -> Result<(), CliError> {
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

    let layout_name = args.layout.as_deref().unwrap_or("default");
    let layout_parsed = if let Some(l_str) = job.definition.layouts.get(layout_name) {
        keyforge_adapter::conversion::parse_layout_string(
            l_str,
            session.engine.key_count(),
            &session.registry,
        )
        .map_err(|e| CliError::Other(format!("Parse Error: {e}")))?
    } else {
        keyforge_adapter::conversion::parse_layout_string(
            layout_name,
            session.engine.key_count(),
            &session.registry,
        )
        .map_err(|e| CliError::Other(format!("Parse Error: {e}")))?
    };

    let report = session
        .engine
        .analyze(&layout_parsed)
        .map_err(|e| CliError::Other(format!("Analysis Error: {e}")))?;
    println!(
        "{}",
        serde_json::to_string_pretty(&report)
            .map_err(|e| CliError::Other(format!("JSON Error: {e}")))?
    );
    Ok(())
}
