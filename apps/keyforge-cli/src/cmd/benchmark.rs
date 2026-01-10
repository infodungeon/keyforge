// apps/keyforge-cli/src/cmd/benchmark.rs

use clap::Args;
use keyforge_protocol::JobConfig;
use crate::runner::AgentRunner;

#[derive(Args, Debug, Clone)]
pub struct BenchmarkArgs {
    #[command(flatten)]
    pub config: crate::cli_args::config::ConfigArgs,
    #[arg(long, default_value_t = 100_000)]
    pub iterations: usize,
    #[command(flatten)]
    pub shared: crate::cmd::shared::SharedArgs,
}

pub fn run(args: BenchmarkArgs, runner: AgentRunner, job_config: JobConfig) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("📊 Benchmarking via Agent ({} iterations)...", args.iterations);
    runner.run_benchmark(&job_config, args.iterations)?;
    Ok(())
}
