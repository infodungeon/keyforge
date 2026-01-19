// apps/keyforge-hive/src/api/mod.rs

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

/// Admin and maintenance endpoints.
pub mod admin;
/// Layout analysis and validation endpoints.
pub mod analysis;
/// User authentication and token management.
pub mod auth;
/// Job lifecycle and management.
pub mod jobs;
/// Cluster metrics and prometheus scraping.
pub mod metrics;
/// Worker node management.
pub mod nodes;
/// Optimization result queries.
pub mod results;
/// General layout submission management.
pub mod submission;
/// Input validation helpers.
pub mod validation;
/// Real-time communication via `WebSockets`.
pub mod ws;

use crate::state::AppState;
use axum::Router;
use std::sync::Arc;
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::PeerIpKeyExtractor, GovernorLayer,
};

/// Returns the router for layout analysis.
pub fn analysis_routes() -> Router<Arc<AppState>> {
    Router::new().route(
        "/analysis/validate",
        axum::routing::post(analysis::validate_layout),
    )
}

/// Returns the router for the public user registration endpoint.
pub fn auth_routes() -> Router<Arc<AppState>> {
    // Strict Limit: 1 request per second per IP for key generation
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(1)
            .burst_size(2)
            .key_extractor(PeerIpKeyExtractor)
            .finish()
            .unwrap_or_else(|| {
                tracing::error!("Failed to initialize auth governor, using fallback");
                #[allow(clippy::expect_used)]
                GovernorConfigBuilder::default()
                    .finish()
                    .expect("fallback governor must be valid")
            }),
    );

    // PUBLIC ROUTE: Register
    Router::new()
        .route("/auth/register", axum::routing::post(auth::register))
        .layer(GovernorLayer::new(governor_conf))
}

/// Returns the router for authenticated authentication endpoints (e.g., API key generation).
pub fn protected_auth_routes() -> Router<Arc<AppState>> {
    // PROTECTED ROUTE: Add Key (Requires Auth Middleware)
    Router::new().route("/auth/keys", axum::routing::post(auth::generate_key))
}

/// Returns the router for administrative and maintenance tasks.
pub fn admin_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/stats", axum::routing::get(admin::get_admin_stats))
        .route("/reload-config", axum::routing::post(admin::reload_config))
        .route("/backup", axum::routing::get(admin::backup_db))
        .route("/cache/clear", axum::routing::post(admin::clear_cache))
}
