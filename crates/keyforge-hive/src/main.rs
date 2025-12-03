use clap::Parser;
use keyforge_core::keycodes::KeycodeRegistry;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

// Module Declarations
mod db;
mod queue; // NEW: Must be declared to be used in state
mod routes;
mod state; // NEW: Must be declared
mod store; // NEW: Must be declared

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

    // Collapsed Path Resolution (Clippy Clean)
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

    // Initialize State (which initializes Store and Queue)
    let state = Arc::new(AppState::new(pool, registry));

    let app = routes::system_routes()
        .merge(routes::job_routes())
        .merge(routes::result_routes())
        .route("/manifest", axum::routing::get(routes::sync::get_manifest))
        .nest_service("/data", ServeDir::new(&data_path))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    info!("🚀 Hive listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
