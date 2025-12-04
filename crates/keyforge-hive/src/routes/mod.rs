// ===== keyforge/crates/keyforge-hive/src/routes/mod.rs =====
pub mod jobs;
pub mod nodes; // NEW: Declare the module
pub mod results;
pub mod submission;
pub mod sync;
pub mod system;

use crate::state::AppState;
use axum::Router;
use std::sync::Arc;

pub fn system_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", axum::routing::get(system::root))
        .route("/health", axum::routing::get(system::health))
}

pub fn job_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/jobs", axum::routing::post(jobs::register))
        .route("/jobs/queue", axum::routing::get(jobs::get_queue))
        .route(
            "/jobs/{job_id}/population",
            axum::routing::get(jobs::get_population),
        )
}

pub fn result_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/results", axum::routing::post(results::submit))
        .route(
            "/submissions",
            axum::routing::post(submission::submit_layout).get(submission::list_submissions),
        )
        // NEW: Wire up the node registration endpoint
        .route("/nodes/register", axum::routing::post(nodes::register_node))
}
