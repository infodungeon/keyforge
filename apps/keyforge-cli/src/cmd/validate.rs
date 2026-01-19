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
    let options = keyforge_runner::RunnerOptions {
        keycodes_file: "keycodes.json".into(),
        ..Default::default()
    };
    let job = build_job_config(loader, &args.shared, args.config.clone())
        .await
        .map_err(|e| CliError::Other(format!("Failed to build job: {e}")))?;
    let session = keyforge_runner::OptimizationRunner::prepare_session(loader, &job, &options)
        .await
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
