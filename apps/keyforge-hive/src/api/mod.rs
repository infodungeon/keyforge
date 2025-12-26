pub mod admin;
pub mod analysis;
pub mod auth;
pub mod corpus;
pub mod jobs;
pub mod metrics;
pub mod nodes;
pub mod results;
pub mod submission;
pub mod sync;
pub mod system;
pub mod user;
pub mod validation;
pub mod ws;

use crate::state::AppState;
use axum::Router;
use std::sync::Arc;
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::PeerIpKeyExtractor, GovernorLayer,
};

pub fn analysis_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/analysis/validate",
            axum::routing::post(analysis::validate_layout),
        )
        .route("/api/corpus/*name", axum::routing::get(corpus::get_corpus))
}

pub fn system_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", axum::routing::get(system::root))
        .route("/health", axum::routing::get(system::health))
        .route(
            "/api/keyboards/:name",
            axum::routing::get(system::get_keyboard),
        )
        .route("/ws", axum::routing::get(ws::handler))
        .route("/sys/metrics", axum::routing::get(metrics::get_metrics))
        .route("/sys/status", axum::routing::get(metrics::get_system_status))
        // Optimized Listing Endpoints
        .route("/api/keyboards", axum::routing::get(system::list_keyboards))
        .route("/api/corpora", axum::routing::get(system::list_corpora))
        .route("/api/costs", axum::routing::get(system::list_costs))
        .route(
            "/api/keymap_extras",
            axum::routing::get(system::list_keymap_extras),
        )
}

pub fn auth_routes() -> Router<Arc<AppState>> {
    // Strict Limit: 1 request per second per IP for key generation
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(1)
            .burst_size(2)
            .key_extractor(PeerIpKeyExtractor)
            .finish()
            .expect("valid auth governor config"),
    );

    // PUBLIC ROUTE: Register
    Router::new()
        .route("/auth/register", axum::routing::post(auth::register))
        .layer(GovernorLayer {
            config: governor_conf,
        })
}

pub fn protected_auth_routes() -> Router<Arc<AppState>> {
    // PROTECTED ROUTE: Add Key (Requires Auth Middleware)
    Router::new().route("/auth/keys", axum::routing::post(auth::generate_key))
}

pub fn admin_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/stats", axum::routing::get(admin::get_admin_stats))
        .route("/reload-config", axum::routing::post(admin::reload_config))
        .route("/backup", axum::routing::get(admin::backup_db))
        .route("/cache/clear", axum::routing::post(admin::clear_cache))
}
