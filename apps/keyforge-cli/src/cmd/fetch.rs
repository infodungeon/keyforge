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
use std::path::Path;
use crate::constants::DEFAULT_HIVE_URL;

#[derive(Args, Debug, Clone)]
pub struct FetchArgs {
    #[command(subcommand)]
    pub command: FetchCommands,

    #[arg(long, default_value = DEFAULT_HIVE_URL)]
    pub hive: String,
}

#[derive(Subcommand, Debug, Clone)]
pub enum FetchCommands {
    /// Download a keyboard definition
    Keyboard { name: String },
    /// Download a corpus bundle
    Corpus { name: String },
    /// Download a cost matrix
    Cost { name: String },
}

pub async fn run(args: FetchArgs, root: &Path) -> Result<(), CliError> {
    let config = keyforge_infra::net::client::ClientConfig {
        base_url: args.hive.clone(),
        secret: None,
        ..Default::default()
    };
    let client = keyforge_infra::HiveClient::new(config)
        .map_err(|e| CliError::Other(format!("Failed to create client: {}", e)))?;

    let manager = keyforge_infra::AssetManager::new(client, root.to_path_buf());
    match args.command {
        FetchCommands::Keyboard { name } => manager.ensure_keyboard(&name).await.map(|_| ()),
        FetchCommands::Corpus { name } => manager.ensure_corpus(&name, None).await.map(|_| ()),
        FetchCommands::Cost { name } => manager.ensure_cost_matrix(&name).await.map(|_| ()),
    }
    .map_err(|e| CliError::Other(format!("Fetch failed: {}", e)))?;

    eprintln!("✅ Fetch successful.");
    Ok(())
}
