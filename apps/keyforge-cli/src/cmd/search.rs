// apps/keyforge-cli/src/cmd/search.rs

use clap::Args;

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
