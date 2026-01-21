// apps/keyforge-cli/src/cmd/init.rs

use crate::constants::{DEFAULT_DATA_DIR, DEFAULT_HIVE_URL};
use crate::error::CliError;
use clap::Args;
use keyforge_infra::init::{ensure_dir, USER_WORKSPACE_DIRS};
use std::path::PathBuf;

#[derive(Args, Debug, Clone)]
pub struct InitArgs {
    #[arg(default_value = ".")]
    pub path: PathBuf,

    #[arg(long, default_value = DEFAULT_HIVE_URL)]
    pub hive: String,

    #[arg(long)]
    pub asset_url: Option<String>,
}

pub async fn run(args: InitArgs) -> Result<(), CliError> {
    let root = args.path.join(DEFAULT_DATA_DIR);
    eprintln!("🚀 Initializing KeyForge Workspace at {}", root.display());

    for d in USER_WORKSPACE_DIRS {
        ensure_dir(&root, d)
            .map_err(|e| CliError::Workspace(format!("Failed to create {d}: {e}")))?;
    }

    let asset_url = args.asset_url.unwrap_or_else(|| {
        // Heuristic: If hive is https://host:3000, assets is usually https://host:3001
        if let Some(pos) = args.hive.rfind(':') {
            if let Ok(port) = args.hive[pos + 1..].parse::<u32>() {
                return format!("{}:{}", &args.hive[..pos], port + 1);
            }
        }
        "http://localhost:3001".to_string()
    });

    eprintln!("🌐 Connecting to Hive at {}...", args.hive);
    let config = keyforge_infra::net::client::ClientConfig {
        api_url: args.hive.clone(),
        asset_url,
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
