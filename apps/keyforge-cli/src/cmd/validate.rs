// apps/keyforge-cli/src/cmd/validate.rs

use clap::Args;

#[derive(Args, Debug, Clone)]
pub struct ValidateArgs {
    #[command(flatten)]
    pub config: crate::cli_args::config::ConfigArgs,

    #[arg(short = 'T', long)]
    pub time: Option<u64>,

    #[command(flatten)]
    pub shared: crate::cmd::shared::SharedArgs,

    /// Name of the layout to validate (must exist in the keyboard definition)
    #[arg(long)]
    pub layout: Option<String>,
}
