// apps/keyforge-cli/src/cmd/init.rs

use crate::error::CliError;
use clap::Args;
use keyforge_infra::init::{ensure_dir, USER_WORKSPACE_DIRS};
use std::path::PathBuf;
use crate::constants::{DEFAULT_HIVE_URL, DEFAULT_DATA_DIR};

#[derive(Args, Debug, Clone)]
pub struct InitArgs {
    #[arg(default_value = ".")]
    pub path: PathBuf,

    #[arg(long, default_value = DEFAULT_HIVE_URL)]
    pub hive: String,

    #[arg(long, default_value = "http://localhost:3001")]
    pub asset_url: String,
}

pub async fn run(args: InitArgs) -> Result<(), CliError> {
    let root = args.path.join(DEFAULT_DATA_DIR);
    eprintln!("🚀 Initializing KeyForge Workspace at {root:?}");

    for d in USER_WORKSPACE_DIRS {
        ensure_dir(&root, d)
            .map_err(|e| CliError::Workspace(format!("Failed to create {d}: {e}")))?;
    }

    eprintln!("🌐 Connecting to Hive at {}...", args.hive);
    let config = keyforge_infra::net::client::ClientConfig {
        api_url: args.hive.clone(),
        asset_url: args.asset_url.clone(),
        secret: None,
        ..Default::default()
    };
    let client = match keyforge_infra::HiveClient::new(config) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("⚠️  Could not connect to Hive: {e}. Skipping downloads.");
            eprintln!("✅ Workspace initialized (offline mode).");
            return Ok(());
        }
    };

    match keyforge_infra::net::sync::bootstrap_essentials(&client, &root).await {
        Ok(files) => {
            eprintln!("✅ Downloaded {} essential assets.", files.len());
        }
        Err(e) => {
            eprintln!("⚠️  Bootstrap failed: {e}");
        }
    }

    eprintln!("✅ Workspace initialized.");
    Ok(())
}
