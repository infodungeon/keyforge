// apps/keyforge-cli/src/cmd/search.rs

use clap::Args;
use keyforge_protocol::JobConfig;
use crate::runner::AgentRunner;

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

pub fn run(args: SearchArgs, runner: AgentRunner, job_config: JobConfig) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("🔎 Delegating optimization to Agent...");
    
    // Note: The runner handles spawning the agent which will output the results directly.
    runner.run_search(&job_config, args.threads)?;
    
    Ok(())
}
