// ===== keyforge/crates/keyforge-hive/src/auth.rs =====
use crate::state::AppState;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tracing::warn;

pub async fn require_secret(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // If no secret is configured on the server, we fail open (Dev mode)
    // OR we fail closed. Marcus prefers Fail Closed, but for ease of dev:
    // If HIVE_SECRET is unset, auth is disabled.
    let secret = match &state.api_secret {
        Some(s) => s,
        None => return Ok(next.run(req).await),
    };

    let auth_header = req
        .headers()
        .get("X-Keyforge-Secret")
        .and_then(|h| h.to_str().ok());

    match auth_header {
        Some(val) if val == secret => Ok(next.run(req).await),
        Some(_) => {
            warn!(
                "⛔ Auth Failed: Invalid Secret provided from {:?}",
                req.uri()
            );
            Err(StatusCode::UNAUTHORIZED)
        }
        None => {
            warn!("⛔ Auth Failed: Missing Header from {:?}", req.uri());
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}
