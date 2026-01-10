// apps/keyforge-hive/src/lib.rs

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


//! # KeyForge Hive
//!
//! The central coordination server for KeyForge. This crate implements the 
//! API server, job queue, and result aggregation logic.

use axum::{
    extract::{ConnectInfo, Request, State},
    http::{Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    Router,
};
use governor::{clock::DefaultClock, state::keyed::DefaultKeyedStateStore, Quota, RateLimiter};
use std::env;
use std::net::{IpAddr, SocketAddr};
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{info, warn, Level};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub(crate) mod api;
pub(crate) mod api_docs;
pub(crate) mod features;
pub(crate) mod auth;
/// Self-healing and bootstrap logic for system assets.
pub mod bootstrap;
/// Global and local caching mechanisms.
pub mod cache;
pub(crate) mod commands;
/// Application configuration and environment variable loading.
pub mod config;
/// Background jobs and periodic tasks.
pub mod cron;
pub(crate) mod error;
/// Infrastructure layer including database and message queue.
pub mod infra;
pub(crate) mod models;
pub(crate) mod monitor;
/// Telemetry, logging, and metrics collection.
pub mod observability;
pub(crate) mod services;
/// Global application state and configuration.
pub mod state;

pub use state::AppState;
pub use services::verification::VerificationService;

/// A keyed rate limiter used for general API traffic management.
pub type GlobalLimiter = RateLimiter<IpAddr, DefaultKeyedStateStore<IpAddr>, DefaultClock>;
/// A stricter keyed rate limiter for sensitive or compute-intensive endpoints.
pub type StrictLimiter = RateLimiter<IpAddr, DefaultKeyedStateStore<IpAddr>, DefaultClock>;

/// Shared state for rate limiting, containing global and strict limiters.
#[derive(Clone)]
pub struct RateLimitState {
    /// General rate limiter for all public endpoints.
    pub global: Arc<GlobalLimiter>,
    /// Stricter rate limiter for sensitive or expensive endpoints (e.g., job registration).
    pub strict: Arc<StrictLimiter>,
}

/// Constructs the main Axum application router.
pub fn create_app(state: Arc<AppState>, config: &config::AppConfig, _data_path: PathBuf) -> Router {
    // --- CORS ---
    let cors_origins = &config.cors_origins;
    let cors = if cors_origins == "*" {
        info!("🔓 CORS: Explicitly Permissive Mode (*)");
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST, Method::DELETE])
            .allow_headers(Any)
    } else if cors_origins.is_empty() {
        info!("🔒 CORS: Dev Mode (allowing localhost:5173, localhost:1420, tauri://localhost)");
        let dev_origins: Vec<axum::http::HeaderValue> = [
            "http://localhost:5173",
            "http://localhost:1420",
            "tauri://localhost",
        ]
        .iter()
        .filter_map(|s| s.parse().ok())
        .collect();

        CorsLayer::new()
            .allow_origin(dev_origins)
            .allow_methods([Method::GET, Method::POST, Method::DELETE])
            .allow_headers(Any)
    } else {
        let origins: Vec<_> = cors_origins
            .split(',')
            .filter_map(|s| {
                let trimmed = s.trim();
                match trimmed.parse() {
                    Ok(v) => Some(v),
                    Err(e) => {
                        warn!("Ignoring invalid CORS origin '{}': {}", trimmed, e);
                        None
                    }
                }
            })
            .collect();
        info!("🔒 CORS: Restricted to {:?}", origins);
        CorsLayer::new()
            .allow_origin(AllowOrigin::list(origins))
            .allow_methods([Method::GET, Method::POST, Method::DELETE])
            .allow_headers(Any)
    };

    // --- RATE LIMITING CONFIG ---
    let limits = &config.rate_limits;

    let rate_limit_state = RateLimitState {
        global: Arc::new(RateLimiter::keyed(
            Quota::per_second(NonZeroU32::new(limits.limit_per_sec.max(1)).unwrap_or(NonZeroU32::MIN))
                .allow_burst(NonZeroU32::new(limits.limit_burst.max(1)).unwrap_or(NonZeroU32::MIN)),
        )),
        strict: Arc::new(RateLimiter::keyed(
            Quota::per_second(NonZeroU32::new(limits.strict_limit_per_sec.max(1)).unwrap_or(NonZeroU32::MIN))
                .allow_burst(NonZeroU32::new(limits.strict_limit_burst.max(1)).unwrap_or(NonZeroU32::MIN)),
        )),
    };

    // --- ROUTES ---
    let secure_routes = Router::new()
        .route(
            "/jobs",
            axum::routing::post(features::register_job::handle).layer(middleware::from_fn_with_state(
                rate_limit_state.clone(),
                strict_rate_limit_middleware,
            )),
        )
        .route("/jobs/queue", axum::routing::get(features::get_queue::handle))
        .route(
            "/jobs/{job_id}/population",
            axum::routing::get(features::get_population::handle),
        )
        .route(
            "/jobs/{job_id}",
            axum::routing::delete(features::cancel_job::handle),
        )
        .route("/results", axum::routing::post(features::submit_result::handle))
        .route(
            "/nodes/register",
            axum::routing::post(features::register_node::handle),
        )
        .route(
            "/submissions",
            axum::routing::post(features::submit_layout::handle),
        )
        .route("/user/nuke", axum::routing::post(features::nuke_user::handle))
        .merge(api::protected_auth_routes())
        .nest("/admin", api::admin_routes())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_secret,
        ));

    let public_routes = features::system::system_routes()
        .merge(api::analysis_routes())
        .merge(api::auth_routes())
        .route(
            "/submissions",
            axum::routing::get(features::list_submissions::handle),
        )
        .route(
            "/data/config.json",
            axum::routing::get(features::system::get_app_config),
        );

    let body_limit = env::var("MAX_JSON_BODY_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1024 * 1024);

    let app_routes = public_routes
        .merge(secure_routes)
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    DefaultMakeSpan::new()
                        .include_headers(true)
                        .level(Level::INFO),
                )
                .on_response(
                    DefaultOnResponse::new()
                        .include_headers(true)
                        .level(Level::INFO),
                ),
        )
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(RequestBodyLimitLayer::new(body_limit))
        .layer(middleware::from_fn_with_state(
            rate_limit_state,
            global_rate_limit_middleware,
        ))
        .route(
            "/jobs/{job_id}/status",
            axum::routing::get(features::get_job_status::handle),
        )
        .with_state(state);

    Router::new()
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", api_docs::ApiDoc::openapi()))
        .merge(app_routes)
}

// --- MIDDLEWARE ---

async fn global_rate_limit_middleware(
    State(limiter): State<RateLimitState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request,
    next: Next,
) -> Response {
    let ip = addr.ip();

    // Whitelist Localhost (IPv4 & IPv6)
    if ip.is_loopback() {
        return next.run(req).await;
    }

    // Check Global Limit
    if limiter.global.check_key(&ip).is_err() {
        warn!("Rate Limit Exceeded (Global) for IP: {}", ip);
        return (StatusCode::TOO_MANY_REQUESTS, "Too Many Requests").into_response();
    }

    next.run(req).await
}

async fn strict_rate_limit_middleware(
    State(limiter): State<RateLimitState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request,
    next: Next,
) -> Response {
    let ip = addr.ip();

    // Whitelist Localhost
    if ip.is_loopback() {
        return next.run(req).await;
    }

    // Check Strict Limit
    if limiter.strict.check_key(&ip).is_err() {
        warn!("Rate Limit Exceeded (Strict) for IP: {}", ip);
        return (StatusCode::TOO_MANY_REQUESTS, "Too Many Requests").into_response();
    }

    next.run(req).await
}
