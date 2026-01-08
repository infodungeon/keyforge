// apps/keyforge-cli/src/cmd/shared.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


use clap::Args;

/// Flags shared only by commands that actually load a physics session.
#[derive(Args, Debug, Clone)]
pub struct SharedArgs {
    /// Keyboard name or path (e.g. "ortho_30", "ansi_104").
    /// Defaults to "ortho_30" if not specified.
    #[arg(short = 'k', long, value_parser = crate::cli_args::parse_keyboard, default_value = "ortho_30", help = "Keyboard name (e.g. 'ortho_30') or file path. Absolute paths and paths relative to CWD or the workspace 'keyboards/' directory are supported.")]
    pub keyboard: String,

    /// Cost-matrix JSON file used for biometric scoring.
    #[arg(short, long, value_parser = crate::cli_args::parse_cost, default_value = "default_costmatrix.json", help = "Path to the cost matrix JSON file. Supports absolute paths and relative paths (checked in CWD and 'keyboards/').")]
    pub cost: String,

    /// Corpus identifiers to load for frequency analysis.
    /// Can be specified multiple times.
    #[arg(
        long,
        default_value = "text/en_std",
        help = "Corpus source identifier (e.g. 'text/en_std') or path. Can use 'name:weight' format."
    )]
    pub corpus: Vec<String>,

    /// Optional path to a custom weights JSON file to override defaults.
    #[arg(
        long,
        help = "Path to a JSON file containing specific scoring weights overrides. Supports absolute and relative paths."
    )]
    pub weights: Option<String>,

    /// Keycodes definition file.
    #[arg(
        long,
        default_value = "keycodes.json",
        help = "Path to the keycodes definition file."
    )]
    pub keycodes: String,

    /// Physical-key constraints.
    /// Format: "INDEX:KEYCODE,..." (e.g. "3:Q,7:W")
    #[arg(long, value_parser = crate::cli_args::parse_key_constraint, value_delimiter = ',', help = "Force specific keys to specific physical indices. Format: 'INDEX:KEY_LABEL'.")]
    pub pinned_keys: Vec<keyforge_model::KeyConstraint>,
}
