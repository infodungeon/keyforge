// apps/keyforge-hive/src/main.rs

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


//! # KeyForge Hive Binary
//!
//! The main entry point for the KeyForge Hive server. This executable 
//! initializes the application state, starts the Axum HTTP server, 
//! and begins background maintenance tasks.

use axum_server::tls_rustls::RustlsConfig;
use clap::{Parser, Subcommand};
use keyforge_hive::{
    bootstrap::HiveBootstrapConfig,
    create_app, cron,
    infra::{db, tui},
    observability,
    state::AppState,
};
use keyforge_infra::init::{ensure_dir, USER_RUNTIME_DIRS};
use keyforge_protocol::AssetManifestEntry;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};
use walkdir::WalkDir;

#[derive(Parser)]
struct Args {
    #[arg(long, env = "KEYFORGE_BOOTSTRAP")]
    bootstrap: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(long, short, env = "KEYFORGE_DATA_DIR")]
    data: Option<PathBuf>,

    #[arg(
        long,
        env = "DATABASE_URL",
        default_value = "postgres://keyforge:forge_password@localhost:5432/keyforge_hive"
    )]
    db: String,

    #[arg(long, env = "PORT", default_value_t = 3000)]
    port: u16,

    #[arg(long, env = "TLS_CERT")]
    tls_cert: Option<PathBuf>,

    #[arg(long, env = "TLS_KEY")]
    tls_key: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    Serve,
    Monitor {
        #[arg(long, default_value = "http://localhost:3000")]
        url: String,
    },
}

/// Populates the Valkey global asset cache from the local filesystem data root.
async fn hydrate_valkey(coordinator: &keyforge_infra::DistributedCoordinator, root: &Path) {
    info!("�� Hydrating Valkey from local system assets...");
    let system_root = root.join("system");
    let walker = WalkDir::new(&system_root).follow_links(true);

    let mut count = 0;
    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path();
            if let Ok(rel) = path.strip_prefix(&system_root) {
                let key_path = rel.to_string_lossy().replace('\\', "/");
                let valkey_key = format!("asset:blob:{}", key_path);

                if let Ok(content) = tokio::fs::read(path).await {
                    let size = content.len() as u64;
                    // Calculate Hash for Manifest
                    let hash = match keyforge_infra::util::common::calculate_file_hash(path) {
                        Ok(h) => h,
                        Err(_) => "unknown".to_string(),
                    };

                    // 1. Upload Blob (if missing)
                    if coordinator.get_bin(&valkey_key).await.unwrap_or(None).is_none() {
                        if let Err(e) = coordinator.set_bin(&valkey_key, &content).await {
                            warn!("Failed to upload {}: {}", key_path, e);
                        } else {
                            count += 1;
                        }
                    }

                    // 2. Update Manifest Entry (Always, to ensure consistency)
                    let entry = AssetManifestEntry {
                        id: key_path.clone(),
                        hash,
                        size_bytes: size,
                        last_updated: chrono::Utc::now().timestamp() as u64,
                    };
                    let _ = coordinator.set_manifest_entry(&entry).await;
                }
            }
        }
    }
    info!("✅ Hydration Complete: {} new assets uploaded.", count);
}

// Handler for clean shutdown in HTTP mode
/// Signal handler for clean shutdown in HTTP mode.
async fn shutdown_signal(state: Arc<AppState>) {
    tokio::signal::ctrl_c().await.ok();
    info!("🛑 Signal received, initiating graceful shutdown...");
    state.queue.shutdown().await;
    info!("👋 Shutdown complete.");
}

// Handler for clean shutdown in TLS mode (axum-server)
// FIX: Added <SocketAddr> generic
/// Signal handler for clean shutdown in TLS mode.
async fn shutdown_signal_axum(handle: axum_server::Handle<SocketAddr>, state: Arc<AppState>) {
    tokio::signal::ctrl_c().await.ok();
    info!("🛑 Signal received (TLS), initiating graceful shutdown...");
    // Stop accepting new connections, wait up to 30s for active requests
    handle.graceful_shutdown(Some(Duration::from_secs(30)));
    // Flush the WriteQueue
    state.queue.shutdown().await;
    info!("👋 Shutdown complete.");
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    match args.command.unwrap_or(Commands::Serve) {
        Commands::Monitor { url } => {
            let secret = std::env::var("HIVE_SECRET").ok();
            if let Err(e) = tui::run_monitor(url, secret).await {
                eprintln!("TUI Error: {}", e);
            }
        }
        Commands::Serve => {
            observability::init_tracing();
            info!("🐝 KeyForge Hive is initializing...");

            // Load Configuration Check
            let config = match keyforge_hive::config::AppConfig::load_from_env() {
                Ok(c) => c,
                Err(e) => {
                    error!("FATAL: Configuration Error: {}", e);
                    std::process::exit(1);
                }
            };
            
            // Allow override of DB URL from CLI args if provided (though env is preferred)
            // Note: Args default matches "postgres://..." so we only use args.db if it differs from default OR if we want CLI priority.
            // But AppConfig enforces DATABASE_URL existence. Let's stick to the AppConfig as the source of truth, 
            // but for backward compatibility, if CLI arg is provided and specific, we might warn. 
            // For now, let's use the loaded config.

            let pool = match db::try_init_db(&config.database_url).await {
                Ok(p) => p,
                Err(e) => {
                    error!("FATAL: Database initialization failed: {}", e);
                    std::process::exit(1);
                }
            };

            let bootstrap_path = args.bootstrap.clone().or_else(|| {
                let p = PathBuf::from(HiveBootstrapConfig::DEFAULT_PATH);
                p.exists().then_some(p)
            });

            let file_config = if let Some(p) = bootstrap_path {
                match HiveBootstrapConfig::load(&p) {
                    Ok(cfg) => {
                        info!("Using bootstrap config: {:?}", p);
                        Some(cfg)
                    }
                    Err(e) => {
                        error!("FATAL: Failed to load bootstrap config: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                None
            };

            let resolved_path = args
                .data
                .clone()
                .or_else(|| file_config.map(|c| c.data_root))
                .unwrap_or_else(|| PathBuf::from("."));

            let data_path = std::fs::canonicalize(&resolved_path).unwrap_or(resolved_path);

            info!("🐝 Data root: {:?}", data_path);

            for d in USER_RUNTIME_DIRS {
                if let Err(e) = ensure_dir(&data_path, d) {
                    error!("FATAL: Failed to create runtime directory {}: {}", d, e);
                    std::process::exit(1);
                }
            }

            let server_key = config.server_key.clone().unwrap_or_else(|| {
                warn!("⚠️ No HIVE_SERVER_KEY configured. Generating ephemeral identity.");
                uuid::Uuid::new_v4().to_string()
            });
            
            // Init State
            let state = Arc::new(AppState::new(pool, data_path.clone(), server_key, config.clone()).await);

            // 2. HYDRATION (Self-Seeding)
            hydrate_valkey(&state.coordinator, &data_path).await;

            let job_repo_arc = Arc::new(state.jobs.repo.clone());
            let node_repo_arc = Arc::new(state.nodes.clone());
            let result_repo_arc = Arc::new(state.results.clone());

            tokio::spawn(cron::start_cron_jobs(
                job_repo_arc,
                node_repo_arc,
                result_repo_arc,
            ));

            let app = create_app(state.clone(), &config, data_path);
            let addr = SocketAddr::from(([0, 0, 0, 0], args.port));

            if let (Some(cert), Some(key)) = (args.tls_cert, args.tls_key) {
                info!("🚀 Hive listening on {} (TLS Enabled)", addr);
                let tls_config = match RustlsConfig::from_pem_file(cert, key).await {
                    Ok(c) => c,
                    Err(e) => {
                        error!("FATAL: Failed to load TLS certificates: {}", e);
                        std::process::exit(1);
                    }
                };

                let handle = axum_server::Handle::new();
                
                // Spawn the shutdown signal handler
                tokio::spawn(shutdown_signal_axum(handle.clone(), state.clone()));

                if let Err(e) = axum_server::bind_rustls(addr, tls_config)
                    .handle(handle)
                    .serve(app.into_make_service_with_connect_info::<SocketAddr>())
                    .await
                {
                    error!("FATAL: TLS server error: {}", e);
                    std::process::exit(1);
                }
            } else {
                info!("🚀 Hive listening on {} (HTTP Mode)", addr);
                let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
                
                if let Err(e) = axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
                    .with_graceful_shutdown(shutdown_signal(state.clone()))
                    .await 
                {
                    error!("FATAL: Server error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}
