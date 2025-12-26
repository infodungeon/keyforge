use crate::error::CliError;
use clap::{Args, Subcommand};
use std::path::Path;

#[derive(Args, Debug, Clone)]
pub struct FetchArgs {
    #[command(subcommand)]
    pub command: FetchCommands,

    #[arg(long, default_value = "http://localhost:3000")]
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
    let client = keyforge_infra::HiveClient::new(args.hive.clone(), None)
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
