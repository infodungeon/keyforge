// apps/keyforge-cli/src/cmd/validate.rs

use clap::Args;
use keyforge_protocol::JobConfig;
use crate::runner::AgentRunner;

#[derive(Args, Debug, Clone)]
pub struct ValidateArgs {
    #[command(flatten)]
    pub config: crate::cli_args::config::ConfigArgs,

    #[command(flatten)]
    pub shared: crate::cmd::shared::SharedArgs,

    /// Name of the layout to validate (must exist in the keyboard definition)
    #[arg(long)]
    pub layout: Option<String>,
}

pub fn run(args: ValidateArgs, runner: AgentRunner, job_config: JobConfig) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("🔎 Validating Layout via Agent...");

    let layout_arg = args.layout.ok_or("Layout string is required for validation")?;

    // Try to find the layout by name in the definition
    let layout_str = if let Some(l) = job_config.definition.layouts.get(&layout_arg) {
        l.clone()
    } else {
        // Fallback: treat as raw layout string
        layout_arg
    };
    
    runner.run_validation(&job_config, &layout_str)?;

    Ok(())
}
