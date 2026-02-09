// apps/keyforge-cli/src/cmd/fetch.rs

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

use crate::error::CliError;
use clap::{Args, Subcommand};
use keyforge_infra::net::client::ClientConfig;
use keyforge_infra::AssetManager;
use keyforge_model::types::path::SafePath;

#[derive(Args, Debug, Clone)]
pub struct FetchArgs {
    #[command(subcommand)]
    pub command: FetchCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum FetchCommands {
    /// Fetches a specific keyboard definition from the Hive.
    Keyboard { name: String },
    /// Fetches a specific corpus from the Hive.
    Corpus { name: String },
    /// Fetches a specific cost matrix from the Hive.
    Cost { name: String },
}

pub async fn run(args: FetchArgs, root: &SafePath) -> Result<(), CliError> {
    let common_config = keyforge_infra::config::CommonConfig::default();
    let hive_url = common_config
        .hive_url
        .clone()
        .unwrap_or_else(|| "http://localhost:3000".into());
    let config = ClientConfig {
        api_url: hive_url.clone(),
        asset_url: hive_url.replace(":3000", ":3001"),
        secret: None,
        ..Default::default()
    };

    let client = keyforge_infra::HiveClient::new(config)
        .map_err(|e| CliError::Other(format!("Client error: {e}")))?;
    let manager = AssetManager::new(client, root.clone());

    match args.command {
        FetchCommands::Keyboard { name } => manager.ensure_keyboard(&name).await.map(|_| ()),
        FetchCommands::Corpus { name } => manager.ensure_corpus(&name, None).await,
        FetchCommands::Cost { name } => manager.ensure_cost_matrix(&name).await,
    }
    .map_err(|e| CliError::Other(format!("Fetch failed: {e}")))?;

    println!("✅ Asset fetched successfully.");
    Ok(())
}
