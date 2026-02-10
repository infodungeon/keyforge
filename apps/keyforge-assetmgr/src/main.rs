#![allow(clippy::print_stdout)]
// apps/keyforge-assetmgr/src/main.rs

use clap::{Parser, Subcommand};
use keyforge_assetmgr::is_hidden;
use keyforge_assetmgr::ops::upload_file;
use keyforge_boundary::SafePath;
use keyforge_infra::net::distributed::{DistributedCoordinator, ValkeyDistributedCoordinator};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(
        long,
        env = "KEYFORGE_VALKEY_URL",
        default_value = "redis://127.0.0.1:6379"
    )]
    valkey_url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Seed {
        #[arg(long, env = "KEYFORGE_DATA_DIR", default_value = "data")]
        data_dir: PathBuf,
    },
    Prune {
        #[arg(long, env = "KEYFORGE_DATA_DIR", default_value = "data")]
        data_dir: PathBuf,
    },
    Verify {
        #[arg(long, env = "KEYFORGE_DATA_DIR", default_value = "data")]
        data_dir: PathBuf,
    },
    Watch {
        #[arg(long, env = "KEYFORGE_DATA_DIR", default_value = "data")]
        data_dir: PathBuf,
    },
    List,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    let coordinator: Arc<dyn DistributedCoordinator> = Arc::new(
        ValkeyDistributedCoordinator::new(&args.valkey_url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to Valkey: {e}"))?,
    );

    match args.command {
        Commands::Seed { data_dir } => {
            let system_root = ensure_system_root(&data_dir)?;
            hydrate_valkey(coordinator.as_ref(), &system_root).await;
        }
        Commands::Prune { data_dir } => {
            let system_root = ensure_system_root(&data_dir)?;
            prune_valkey(coordinator.as_ref(), &system_root).await?;
        }
        Commands::Verify { data_dir } => {
            let system_root = ensure_system_root(&data_dir)?;
            verify_integrity(coordinator.as_ref(), &system_root).await?;
        }
        Commands::List => {
            list_assets(coordinator.as_ref()).await?;
        }
        Commands::Watch { data_dir } => {
            let system_root = ensure_system_root(&data_dir)?;
            info!("👀 Watching {:?} for changes...", system_root);
            watch_loop(coordinator, system_root).await?;
        }
    }

    Ok(())
}

fn ensure_system_root(data_dir: &Path) -> anyhow::Result<PathBuf> {
    let root = data_dir.join("system");
    if !root.exists() {
        return Err(anyhow::anyhow!(
            "System root not found at {}",
            root.display()
        ));
    }
    Ok(root)
}

async fn list_assets(coordinator: &dyn DistributedCoordinator) -> anyhow::Result<()> {
    let entries = coordinator
        .get_all_manifest_entries()
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    println!("{:<60} | {:<15} | {:<20}", "ID", "Size", "Updated");
    println!("{:-<60}-+-{:-<15}-+-{:-<20}", "", "", "");

    for (id, hash) in entries {
        println!("{:<60} | {:<15} | {}", id, "???", hash);
    }
    Ok(())
}

async fn hydrate_valkey(coordinator: &dyn DistributedCoordinator, system_root: &Path) {
    info!("🌱 Seeding Valkey from local assets: {:?}", system_root);
    let walker = WalkDir::new(system_root).follow_links(true);
    let mut count = 0;

    for entry in walker.into_iter().filter_map(std::result::Result::ok) {
        if entry.file_type().is_file() {
            if is_hidden(entry.path()) {
                continue;
            }
            if let Err(e) = upload_file(coordinator, system_root, entry.path()).await {
                warn!("Failed to process {:?}: {}", entry.path(), e);
            } else {
                count += 1;
            }
        }
    }
    info!("✅ Seeding Complete: {} files scanned.", count);
}

async fn prune_valkey(
    coordinator: &dyn DistributedCoordinator,
    system_root: &Path,
) -> anyhow::Result<()> {
    info!("✂️ Pruning Valkey orphans...");
    let entries = coordinator
        .get_all_manifest_entries()
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    let mut deleted = 0;

    for (id, _) in entries {
        let local_path = system_root.join(&id);
        if !local_path.exists() {
            info!("🗑️  Orphan found: {}", id);
            warn!("Delete not yet implemented in Coordinator: {}", id);
            deleted += 1;
        }
    }
    info!("✨ Prune Complete. {} orphans identified.", deleted);
    Ok(())
}

async fn verify_integrity(
    coordinator: &dyn DistributedCoordinator,
    system_root: &Path,
) -> anyhow::Result<()> {
    info!("🔍 Verifying Integrity...");
    let entries = coordinator
        .get_all_manifest_entries()
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    let mut mismatches = 0;

    for (id, remote_hash) in entries {
        let p_id = match SafePath::try_from_str(&id) {
            Ok(p) => p,
            Err(e) => {
                error!("❌ Invalid Asset ID {id}: {e}");
                mismatches += 1;
                continue;
            }
        };
        let local_path = SafePath::from_trusted_root(system_root, &p_id);
        if !local_path.as_path().exists() {
            error!("❌ Missing Local: {}", id);
            mismatches += 1;
            continue;
        }

        let local_hash = keyforge_infra::util::common::calculate_file_hash(&local_path)
            .map_err(|e| anyhow::anyhow!(e))?;

        if local_hash != remote_hash {
            error!(
                "❌ Hash Mismatch: {} (Disk: {} vs Valkey: {})",
                id,
                &local_hash[0..8],
                &remote_hash[0..8]
            );
            mismatches += 1;
        }
    }

    if mismatches == 0 {
        info!("✅ Integrity Check Passed.");
    } else {
        error!("🚨 Found {} integrity issues.", mismatches);
        std::process::exit(1);
    }
    Ok(())
}

async fn watch_loop(
    coordinator: Arc<dyn DistributedCoordinator>,
    system_root: PathBuf,
) -> anyhow::Result<()> {
    let (tx, mut rx) = mpsc::channel(100);

    let watcher_root = system_root.clone();
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            let _ = tx.blocking_send(res);
        },
        Config::default(),
    )?;

    watcher.watch(&system_root, RecursiveMode::Recursive)?;

    while let Some(res) = rx.recv().await {
        match res {
            Ok(event) => match event.kind {
                notify::EventKind::Create(_) | notify::EventKind::Modify(_) => {
                    for path in event.paths {
                        if path.is_file() && !is_hidden(&path) {
                            info!("♻️  Change detected: {:?}", path);
                            if let Err(e) =
                                upload_file(coordinator.as_ref(), &watcher_root, &path).await
                            {
                                error!("Sync failed: {}", e);
                            }
                        }
                    }
                }
                notify::EventKind::Remove(_) => {
                    for path in event.paths {
                        warn!(
                            "🗑️  Delete detected (Automatic pruning not enabled for safety): {:?}",
                            path
                        );
                    }
                }
                _ => {}
            },
            Err(e) => error!("Watch error: {:?}", e),
        }
    }
    Ok(())
}
