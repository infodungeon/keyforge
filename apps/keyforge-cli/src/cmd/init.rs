use crate::error::CliError;
use clap::Args;
use keyforge_infra::init::initialize_workspace;
use std::path::PathBuf;

#[derive(Args, Debug, Clone)]
pub struct InitArgs {
    #[arg(default_value = ".")]
    pub path: PathBuf,

    #[arg(long, default_value = "http://localhost:3000")]
    pub hive: String,
}

pub async fn run(args: InitArgs) -> Result<(), CliError> {
    let root = args.path.join("data");
    eprintln!("🚀 Initializing KeyForge Workspace at {:?}", root);

    // 1. Create Structure (Offline)
    initialize_workspace(&root, keyforge_infra::InitMode::Provision)
        .map_err(|e| CliError::Workspace(format!("Initialization failed: {}", e)))?;

    // 2. Download Essentials (Online)
    eprintln!("🌐 Connecting to Hive at {}...", args.hive);
    let client = match keyforge_infra::HiveClient::new(args.hive, None) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("⚠️  Could not connect to Hive: {}. Skipping downloads.", e);
            eprintln!("✅ Workspace initialized (offline mode).");
            return Ok(());
        }
    };

    let manager = keyforge_infra::AssetManager::new(client, root.clone());

    let assets = [
        ("keyboard", "ansi_104"),
        ("keyboard", "corne"),
        ("corpus", "default"),
        ("cost", "cost_matrix.json"),
    ];

    for (kind, name) in assets {
        let res = match kind {
            "keyboard" => manager.ensure_keyboard(name).await,
            "corpus" => manager.ensure_corpus(name, None).await,
            "cost" => manager.ensure_cost_matrix(name).await,
            _ => Ok(PathBuf::new()),
        };

        match res {
            Ok(_) => eprintln!("   ⬇️  Downloaded: {}", name),
            Err(e) => eprintln!("   ⚠️  Failed to download {}: {}", name, e),
        }
    }

    eprintln!("✅ Workspace initialized.");
    Ok(())
}
