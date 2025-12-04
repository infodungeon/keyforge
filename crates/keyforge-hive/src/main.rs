use axum::http::Method;
use clap::Parser;
use keyforge_core::keycodes::KeycodeRegistry;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

// Module Declarations
mod db;
mod queue;
mod routes;
mod state;
mod store;

use crate::state::AppState;

#[derive(Parser)]
struct Args {
    #[arg(long, short, default_value = "data")]
    data: PathBuf,

    #[arg(long, default_value = "sqlite://hive.db")]
    db: String,

    #[arg(long, default_value_t = 3000)]
    port: u16,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    info!("🐝 KeyForge Hive is initializing...");

    let pool = db::init_db(&args.db).await;

    // Collapsed Path Resolution strategy
    let data_path = if args.data.exists() {
        args.data
    } else if Path::new("../data").exists() {
        PathBuf::from("../data")
    } else if Path::new("../../data").exists() {
        PathBuf::from("../../data")
    } else {
        args.data
    };

    info!("📂 Serving static files from: {:?}", data_path);

    let keycodes_path = data_path.join("keycodes.json");
    let registry = if keycodes_path.exists() {
        info!("🔑 Loading Keycodes from {:?}", keycodes_path);
        KeycodeRegistry::load_from_file(&keycodes_path).unwrap_or_else(|e| {
            warn!("Failed to load keycodes: {}. Using defaults.", e);
            KeycodeRegistry::new_with_defaults()
        })
    } else {
        warn!("⚠️ keycodes.json not found. Using defaults.");
        KeycodeRegistry::new_with_defaults()
    };

    let state = Arc::new(AppState::new(pool, registry));

    // --- SECURITY HARDENING ---
    // 1. CORS: Allow Any Origin (Distributed Nodes), but restrict Methods
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    let app = routes::system_routes()
        .merge(routes::job_routes())
        .merge(routes::result_routes())
        .route("/manifest", axum::routing::get(routes::sync::get_manifest))
        // ServeDir handles Range requests automatically, good for large files
        .nest_service("/data", ServeDir::new(&data_path))
        // 2. LAYERS (Order matters: Bottom executes first)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        // 3. DoS Protection: Limit bodies to 64MB (Accommodates large corpora)
        .layer(RequestBodyLimitLayer::new(64 * 1024 * 1024))
        .with_state(state.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    info!("🚀 Hive listening on {}", addr);
    info!("⚠️  WARNING: Server is bound to all interfaces (0.0.0.0). Ensure firewall rules are active.");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    // 4. Graceful Shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(state))
        .await
        .unwrap();
}

/// Listens for SIGTERM/Ctrl+C and flushes the DB queue before exiting
async fn shutdown_signal(state: Arc<AppState>) {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("🛑 Signal received, shutting down...");

    // Flush the WriteQueue to ensure no results are lost
    state.queue.shutdown().await;
}
