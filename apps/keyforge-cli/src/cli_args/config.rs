// apps/keyforge-cli/src/cli_args/config.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use clap::Args;
use keyforge_model::config::definitions::LayoutDefinitionsConfig;
use keyforge_model::config::search::SearchParamsConfig;
use keyforge_model::config::weights::ScoringWeightsConfig;
use keyforge_model::config::{Config, LayoutDefinitions, ScoringWeights, SearchParams};

/// Top-level configuration arguments combining search, weights, and definitions.
#[derive(Args, Debug, Clone)]
pub struct ConfigArgs {
    #[command(flatten)]
    pub search: SearchParamsConfig,
    #[command(flatten)]
    pub weights: ScoringWeightsConfig,
    #[command(flatten)]
    pub defs: LayoutDefinitionsConfig,
}

use std::convert::TryFrom;

impl TryFrom<ConfigArgs> for Config {
    type Error = String;
    fn try_from(args: ConfigArgs) -> Result<Self, Self::Error> {
        let config = Config {
            search: SearchParams::try_from(args.search)?,
            weights: ScoringWeights::try_from(args.weights)?,
            defs: LayoutDefinitions::try_from(args.defs)?,
            pinned_keys: vec![], // Handled via CLI shared args
        };
        Ok(config)
    }
}
