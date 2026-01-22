// apps/keyforge-hive/src/main.rs

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

//! # `KeyForge` Hive Binary
//!
//! The main entry point for the `KeyForge` Hive server. This executable
//! initializes the application state, starts the Axum HTTP server,
//! and begins background maintenance tasks.

use axum_server::tls_rustls::RustlsConfig;
use clap::{Parser, Subcommand};
use keyforge_hive::constants::{
    DEFAULT_DATABASE_URL, DEFAULT_HIVE_PORT, DEFAULT_SHUTDOWN_TIMEOUT_SECS,
};
use keyforge_hive::{
    bootstrap::HiveBootstrapConfig, create_app, cron, infra::db, observability, state::AppState,
};
use keyforge_infra::init::{ensure_dir, USER_RUNTIME_DIRS};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

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
        default_value = DEFAULT_DATABASE_URL
    )]
    db: String,

    #[arg(long, env = "PORT", default_value_t = DEFAULT_HIVE_PORT)]
    port: u16,

    /// Port for the embedded asset server.
    #[arg(long, env = "ASSET_PORT", default_value_t = 3001)]
    asset_port: u16,

    #[arg(long, env = "TLS_CERT")]
    tls_cert: Option<PathBuf>,

    #[arg(long, env = "TLS_KEY")]
    tls_key: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    Serve,
}

// Handler for clean shutdown in HTTP mode
/// Signal handler for clean shutdown in HTTP mode.
async fn shutdown_signal(state: Arc<AppState>) {
    tokio::signal::ctrl_c().await.ok();
    info!("🛑 Signal received, initiating graceful shutdown...");
    state.queue.shutdown();
    info!("👋 Shutdown complete.");
}

// Handler for clean shutdown in TLS mode (axum-server)
/// Signal handler for clean shutdown in TLS mode.
async fn shutdown_signal_axum(handle: axum_server::Handle<SocketAddr>, state: Arc<AppState>) {
    tokio::signal::ctrl_c().await.ok();
    info!("🛑 Signal received (TLS), initiating graceful shutdown...");
    // Stop accepting new connections, wait up to DEFAULT_SHUTDOWN_TIMEOUT_SECS
    handle.graceful_shutdown(Some(Duration::from_secs(DEFAULT_SHUTDOWN_TIMEOUT_SECS)));
    // Flush the WriteQueue
    state.queue.shutdown();
    info!("👋 Shutdown complete.");
}

#[allow(clippy::too_many_lines)]
#[tokio::main]
async fn main() {
    let args = Args::parse();

    match args.command.unwrap_or(Commands::Serve) {
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

            let pool = match db::try_init_db(&config.database_url).await {
                Ok(p) => p,
                Err(e) => {
                    error!("FATAL: Database initialization failed: {}", e);
                    std::process::exit(1);
                }
            };

            // Use resolve_path logic
            let bootstrap_path = args
                .bootstrap
                .clone()
                .unwrap_or_else(HiveBootstrapConfig::resolve_path);

            let file_config = if bootstrap_path.exists() {
                match HiveBootstrapConfig::load(&bootstrap_path) {
                    Ok(cfg) => {
                        info!("Using bootstrap config: {:?}", bootstrap_path);
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
            let state = match AppState::new(pool, data_path.clone(), server_key, config.clone()).await {
                Ok(s) => Arc::new(s),
                Err(e) => {
                    error!("FATAL: Application state initialization failed: {}", e);
                    std::process::exit(1);
                }
            };

            let job_repo_arc = Arc::new(state.jobs.repo.clone());
            let node_repo_arc = Arc::new(state.nodes.clone());
            let result_repo_arc = Arc::new(state.results.clone());

            tokio::spawn(cron::start_cron_jobs(
                job_repo_arc,
                node_repo_arc,
                result_repo_arc,
            ));

            // Start Embedded Asset Server (Configurable Port)
            let asset_provider = state.assets.clone();
            let asset_app = keyforge_assets::create_app(asset_provider);
            let asset_port = args.asset_port;

            tokio::spawn(async move {
                let addr = SocketAddr::from(([0, 0, 0, 0], asset_port));
                info!("🚀 Embedded Asset Server listening on http://{}", addr);
                let listener = match tokio::net::TcpListener::bind(addr).await {
                    Ok(l) => l,
                    Err(e) => {
                        error!(
                            "FATAL: Failed to bind Asset Server port {}: {}",
                            asset_port, e
                        );
                        return;
                    }
                };
                if let Err(e) = axum::serve(listener, asset_app).await {
                    error!("Asset Server crashed: {}", e);
                }
            });

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
                let listener = match tokio::net::TcpListener::bind(addr).await {
                    Ok(l) => l,
                    Err(e) => {
                        error!("FATAL: Failed to bind to {}: {}", addr, e);
                        std::process::exit(1);
                    }
                };

                if let Err(e) = axum::serve(
                    listener,
                    app.into_make_service_with_connect_info::<SocketAddr>(),
                )
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
