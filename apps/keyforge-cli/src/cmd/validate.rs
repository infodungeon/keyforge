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

    let layout_str = args.layout.ok_or("Layout string is required for validation")?;
    
    runner.run_validation(&job_config, &layout_str)?;

    Ok(())
}
