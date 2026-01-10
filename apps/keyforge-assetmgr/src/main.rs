// apps/keyforge-assetmgr/src/main.rs

use clap::{Parser, Subcommand};
use keyforge_infra::DistributedCoordinator;
use keyforge_protocol::AssetManifestEntry;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, env = "KEYFORGE_VALKEY_URL", default_value = "redis://127.0.0.1:6379")]
    valkey_url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Uploads local assets to Valkey (Idempotent).
    Seed {
        #[arg(long, env = "KEYFORGE_DATA_DIR", default_value = "data")]
        data_dir: PathBuf,
    },
    /// Removes assets from Valkey that do not exist locally.
    Prune {
        #[arg(long, env = "KEYFORGE_DATA_DIR", default_value = "data")]
        data_dir: PathBuf,
    },
    /// Checks integrity between Local Disk and Valkey.
    Verify {
        #[arg(long, env = "KEYFORGE_DATA_DIR", default_value = "data")]
        data_dir: PathBuf,
    },
    /// Watches local directory and syncs changes to Valkey in real-time.
    Watch {
        #[arg(long, env = "KEYFORGE_DATA_DIR", default_value = "data")]
        data_dir: PathBuf,
    },
    /// Lists all assets currently in Valkey.
    List,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    let coordinator = Arc::new(DistributedCoordinator::new(&args.valkey_url).await
        .map_err(|e| anyhow::anyhow!("Failed to connect to Valkey: {}", e))?);

    match args.command {
        Commands::Seed { data_dir } => {
            let system_root = ensure_system_root(&data_dir)?;
            hydrate_valkey(&coordinator, &system_root).await;
        }
        Commands::Prune { data_dir } => {
            let system_root = ensure_system_root(&data_dir)?;
            prune_valkey(&coordinator, &system_root).await?;
        }
        Commands::Verify { data_dir } => {
            let system_root = ensure_system_root(&data_dir)?;
            verify_integrity(&coordinator, &system_root).await?;
        }
        Commands::List => {
            list_assets(&coordinator).await?;
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
        return Err(anyhow::anyhow!("System root not found at {:?}", root));
    }
    Ok(root)
}

async fn list_assets(coordinator: &DistributedCoordinator) -> anyhow::Result<()> {
    let entries = coordinator.get_all_manifest_entries().await
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    
    println!("{:<60} | {:<15} | {:<20}", "ID", "Size", "Updated");
    println!("{:-<60}-+-{:-<15}-+-{:-<20}", "", "", "");
    
    for (id, hash) in entries {
        // We need to fetch metadata to get size/updated if we want detailed list
        // For now, we only have hash from the simple map getter.
        // To get full details we'd need to HGETALL key. 
        // Assuming simple list for now.
        println!("{:<60} | {:<15} | {}", id, "???", hash);
    }
    Ok(())
}

async fn hydrate_valkey(coordinator: &DistributedCoordinator, system_root: &Path) {
    info!("🌱 Seeding Valkey from local assets: {:?}", system_root);
    let walker = WalkDir::new(system_root).follow_links(true);
    let mut count = 0;

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if is_hidden(&entry.path()) { continue; }
            if let Err(e) = upload_file(coordinator, system_root, &entry.path()).await {
                warn!("Failed to process {:?}: {}", entry.path(), e);
            } else {
                count += 1;
            }
        }
    }
    info!("✅ Seeding Complete: {} files scanned.", count);
}

async fn prune_valkey(coordinator: &DistributedCoordinator, system_root: &Path) -> anyhow::Result<()> {
    info!("✂️ Pruning Valkey orphans...");
    let entries = coordinator.get_all_manifest_entries().await.map_err(|e| anyhow::anyhow!(e))?;
    let mut deleted = 0;

    for (id, _) in entries {
        let local_path = system_root.join(&id);
        if !local_path.exists() {
            info!("🗑️  Orphan found: {}", id);
            // Delete Manifest Entry
            // Delete Blob
            // (Coordinator needs a delete method, we assume it exists or we implement raw if needed)
            // For now, listing them.
            warn!("Delete not yet implemented in Coordinator: {}", id);
            deleted += 1;
        }
    }
    info!("✨ Prune Complete. {} orphans identified.", deleted);
    Ok(())
}

async fn verify_integrity(coordinator: &DistributedCoordinator, system_root: &Path) -> anyhow::Result<()> {
    info!("🔍 Verifying Integrity...");
    let entries = coordinator.get_all_manifest_entries().await.map_err(|e| anyhow::anyhow!(e))?;
    let mut mismatches = 0;

    for (id, remote_hash) in entries {
        let local_path = system_root.join(&id);
        if !local_path.exists() {
            error!("❌ Missing Local: {}", id);
            mismatches += 1;
            continue;
        }

        let local_hash = keyforge_infra::util::common::calculate_file_hash(&local_path)
            .map_err(|e| anyhow::anyhow!(e))?;

        if local_hash != remote_hash {
            error!("❌ Hash Mismatch: {} (Disk: {} vs Valkey: {})", id, &local_hash[0..8], &remote_hash[0..8]);
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

async fn watch_loop(coordinator: Arc<DistributedCoordinator>, system_root: PathBuf) -> anyhow::Result<()> {
    let (tx, mut rx) = mpsc::channel(100);
    
    let watcher_root = system_root.clone();
    let mut watcher = RecommendedWatcher::new(move |res| {
        let _ = tx.blocking_send(res);
    }, Config::default())?;

    watcher.watch(&system_root, RecursiveMode::Recursive)?;

    while let Some(res) = rx.recv().await {
        match res {
            Ok(event) => {
                match event.kind {
                    notify::EventKind::Create(_) | notify::EventKind::Modify(_) => {
                        for path in event.paths {
                            if path.is_file() && !is_hidden(&path) {
                                info!("♻️  Change detected: {:?}", path);
                                // Debounce could go here
                                if let Err(e) = upload_file(&coordinator, &watcher_root, &path).await {
                                    error!("Sync failed: {}", e);
                                }
                            }
                        }
                    }
                    notify::EventKind::Remove(_) => {
                        for path in event.paths {
                            // Convert to ID and delete
                            warn!("🗑️  Delete detected (Automatic pruning not enabled for safety): {:?}", path);
                        }
                    }
                    _ => {}
                }
            }
            Err(e) => error!("Watch error: {:?}", e),
        }
    }
    Ok(())
}

// --- Helpers ---

fn is_hidden(path: &Path) -> bool {
    path.file_name().and_then(|s| s.to_str()).map(|s| s.starts_with('.')).unwrap_or(false)
}

async fn upload_file(coordinator: &DistributedCoordinator, root: &Path, path: &Path) -> anyhow::Result<()> {
    let rel = path.strip_prefix(root)?;
    let key_path = rel.to_string_lossy().replace('\\', "/");
    let valkey_key = format!("asset:blob:{}", key_path);

    let content = tokio::fs::read(path).await?;
    let size = content.len() as u64;
    let hash = keyforge_infra::util::common::calculate_file_hash(path).map_err(|e| anyhow::anyhow!(e))?;

    // Optimistic check: if manifest matches, assume blob is good (saving bandwidth)
    if let Ok(Some(remote_hash)) = coordinator.get_manifest_hash(&key_path).await {
        if remote_hash == hash {
            // Already synced
            return Ok(());
        }
    }

    coordinator.set_bin(&valkey_key, &content).await.map_err(|e| anyhow::anyhow!(e))?;
    
    let entry = AssetManifestEntry {
        id: key_path.clone(),
        hash,
        size_bytes: size,
        last_updated: chrono::Utc::now().timestamp() as u64,
    };
    coordinator.set_manifest_entry(&entry).await.map_err(|e| anyhow::anyhow!(e))?;
    
    info!("⬆️  Synced: {}", key_path);
    Ok(())
}
