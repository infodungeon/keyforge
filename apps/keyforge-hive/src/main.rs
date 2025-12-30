#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static ALLOC: Jemalloc = Jemalloc;

use axum_server::tls_rustls::RustlsConfig;
use clap::{Parser, Subcommand};
use keyforge_hive::{
    bootstrap::HiveBootstrapConfig,
    create_app, cron,
    infra::{db, tui},
    observability,
    state::AppState,
};
use keyforge_infra::init::initialize_workspace;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

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

            let pool = match db::try_init_db(&args.db).await {
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

            if !resolved_path.exists() {
                error!("FATAL: Data directory does not exist: {:?}", resolved_path);
                std::process::exit(1);
            }

            let data_path = match std::fs::canonicalize(&resolved_path) {
                Ok(p) => p,
                Err(e) => {
                    error!(
                        "FATAL: Failed to canonicalize data root {:?}: {}",
                        resolved_path, e
                    );
                    std::process::exit(1);
                }
            };

            info!("�� Data root: {:?}", data_path);
            if let Err(e) = initialize_workspace(&data_path, keyforge_infra::InitMode::Validate) {
                error!("FATAL: Workspace initialization failed: {}", e);
                std::process::exit(1);
            }

            let server_key = std::env::var("HIVE_SERVER_KEY")
                .unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());
            let state = Arc::new(AppState::new(pool, data_path.clone(), server_key));

            if let Err(e) = state.assets.warm_all().await {
                error!("FATAL: Asset warmup failed: {}", e);
                std::process::exit(1);
            }

            let job_repo_arc = Arc::new(state.jobs.repo.clone());
            let node_repo_arc = Arc::new(state.nodes.clone());
            let result_repo_arc = Arc::new(state.results.clone());

            tokio::spawn(cron::start_cron_jobs(
                job_repo_arc,
                node_repo_arc,
                result_repo_arc,
            ));

            let app = create_app(state.clone(), data_path);
            let addr = SocketAddr::from(([0, 0, 0, 0], args.port));

            if let (Some(cert), Some(key)) = (args.tls_cert, args.tls_key) {
                info!("🚀 Hive listening on {} (TLS Enabled)", addr);
                let config = match RustlsConfig::from_pem_file(cert, key).await {
                    Ok(c) => c,
                    Err(e) => {
                        error!("FATAL: Failed to load TLS certificates: {}", e);
                        std::process::exit(1);
                    }
                };

                let handle = axum_server::Handle::new();
                tokio::spawn(shutdown_signal_axum(handle.clone(), state));

                if let Err(e) = axum_server::bind_rustls(addr, config)
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
                        error!("FATAL: Failed to bind port {}: {}", addr, e);
                        std::process::exit(1);
                    }
                };
                if let Err(e) = axum::serve(
                    listener,
                    app.into_make_service_with_connect_info::<SocketAddr>(),
                )
                .with_graceful_shutdown(shutdown_signal(state))
                .await
                {
                    error!("FATAL: Server error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}

async fn shutdown_signal(state: Arc<AppState>) {
    tokio::signal::ctrl_c().await.ok();
    info!("🛑 Signal received, initiating graceful shutdown...");
    state.queue.shutdown().await;
    info!("👋 Shutdown complete.");
}

async fn shutdown_signal_axum(handle: axum_server::Handle, state: Arc<AppState>) {
    tokio::signal::ctrl_c().await.ok();
    info!("🛑 Signal received (TLS), initiating graceful shutdown...");
    handle.graceful_shutdown(Some(Duration::from_secs(30)));
    state.queue.shutdown().await;
    info!("👋 Shutdown complete.");
}
