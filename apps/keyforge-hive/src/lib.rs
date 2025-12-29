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

pub mod api;
pub mod api_docs;
pub mod features;
pub mod auth;
pub mod bootstrap;
pub mod cache;
pub mod config;
pub mod cron;
pub mod error;
pub mod infra;
pub mod models;
pub mod monitor;
pub mod observability;
pub mod services;
pub mod state;

pub use state::AppState;

// Custom Rate Limiter State
// We use DefaultKeyedStateStore which typically uses DashMap internally for thread-safe key storage.
type GlobalLimiter = RateLimiter<IpAddr, DefaultKeyedStateStore<IpAddr>, DefaultClock>;
type StrictLimiter = RateLimiter<IpAddr, DefaultKeyedStateStore<IpAddr>, DefaultClock>;

#[derive(Clone)]
pub struct RateLimitState {
    global: Arc<GlobalLimiter>,
    strict: Arc<StrictLimiter>,
}

pub fn create_app(state: Arc<AppState>, _data_path: PathBuf) -> Router {
    // --- CORS ---
    let cors_origins = env::var("CORS_ALLOWED_ORIGINS").unwrap_or_default();
    let cors = if cors_origins == "*" {
        info!("🔓 CORS: Explicitly Permissive Mode (*)");
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST, Method::DELETE])
            .allow_headers(Any)
    } else if cors_origins.is_empty() {
        info!("🔒 CORS: Dev Mode (allowing localhost:5173, localhost:1420, tauri://localhost)");
        CorsLayer::new()
            .allow_origin([
                "http://localhost:5173".parse().expect("valid dev origin"),
                "http://localhost:1420".parse().expect("valid dev origin"),
                "tauri://localhost".parse().expect("valid dev origin"),
            ])
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
    let limit_per_sec: u32 = env::var("RATE_LIMIT_PER_SEC")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000); // Default: 1000 req/s (High for TUI/Dev)

    let limit_burst: u32 = env::var("RATE_LIMIT_BURST")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(2000);

    let strict_limit_per_sec: u32 = env::var("STRICT_RATE_LIMIT_PER_SEC")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    let strict_limit_burst: u32 = env::var("STRICT_RATE_LIMIT_BURST")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);

    let rate_limit_state = RateLimitState {
        global: Arc::new(RateLimiter::keyed(
            Quota::per_second(NonZeroU32::new(limit_per_sec.max(1)).unwrap())
                .allow_burst(NonZeroU32::new(limit_burst.max(1)).unwrap()),
        )),
        strict: Arc::new(RateLimiter::keyed(
            Quota::per_second(NonZeroU32::new(strict_limit_per_sec.max(1)).unwrap())
                .allow_burst(NonZeroU32::new(strict_limit_burst.max(1)).unwrap()),
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
        .route("/manifest", axum::routing::get(features::assets::get_manifest))
        .route(
            "/submissions",
            axum::routing::get(features::list_submissions::handle),
        )
        .route(
            "/data/system/*path",
            axum::routing::get(features::assets::get_asset),
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
