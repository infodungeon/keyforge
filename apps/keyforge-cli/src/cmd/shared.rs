// apps/keyforge-cli/src/cmd/shared.rs

use clap::Args;

/// Flags shared only by commands that actually load a physics session.
#[derive(Args, Debug, Clone)]
pub struct SharedArgs {
    /// Keyboard name or path.
    #[arg(short = 'k', long, value_parser = crate::cli_args::parse_keyboard, help = "Keyboard name (e.g. 'ortho_30') or file path.")]
    pub keyboard: Option<String>,

    /// Cost-matrix JSON file used for biometric scoring.
    #[arg(short, long, value_parser = crate::cli_args::parse_cost, help = "Path to the cost matrix JSON file.")]
    pub cost: Option<String>,

    /// Corpus identifiers to load for frequency analysis.
    #[arg(
        long,
        help = "Corpus source identifier (e.g. 'text/en_std') or path."
    )]
    pub corpus: Option<Vec<String>>,

    /// Optional path to a custom weights JSON file to override defaults.
    #[arg(
        long,
        help = "Path to a JSON file containing specific scoring weights overrides."
    )]
    pub weights: Option<String>,

    /// Keycodes definition file.
    #[arg(
        long,
        help = "Path to the keycodes definition file."
    )]
    pub keycodes: Option<String>,

    /// Physical-key constraints.
    /// Format: "INDEX:KEYCODE,..." (e.g. "3:Q,7:W")
    #[arg(long, value_parser = crate::cli_args::parse_key_constraint, value_delimiter = ',', help = "Force specific keys to specific physical indices. Format: 'INDEX:KEY_LABEL'.")]
    pub pinned_keys: Vec<keyforge_model::KeyConstraint>,
}