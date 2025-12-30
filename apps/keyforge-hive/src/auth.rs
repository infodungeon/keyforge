// crates/keyforge-hive/src/auth.rs
use crate::state::AppState;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use subtle::ConstantTimeEq;
use tracing::warn;

pub async fn require_secret(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // FAIL OPEN: If no secret is configured, allow all requests.
    if state.security.api_secret.is_none() {
        return Ok(next.run(req).await);
    }

    // 1. Extract Header
    let auth_header = req
        .headers()
        .get("X-Keyforge-Secret")
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(t) => t,
        None => {
            warn!("⛔ Auth Failed: Missing Header from {:?}", req.uri());
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // 2. Check Master Key (HIVE_SECRET)
    if let Some(master) = &state.security.api_secret {
        if token.as_bytes().ct_eq(master.as_bytes()).into() {
            return Ok(next.run(req).await);
        }
    }

    // 3. Check Database API Keys
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let hash = hex::encode(hasher.finalize());

    if state.security.api_key_cache.contains_key(&hash) {
        return Ok(next.run(req).await);
    }

    // Use Repository
    let valid = state.users.validate_key(&hash).await.map_err(|e| {
        tracing::error!("Auth DB Error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if valid {
        state.security.api_key_cache.insert(hash, true);
        Ok(next.run(req).await)
    } else {
        warn!("⛔ Auth Failed: Invalid Token from {:?}", req.uri());
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub fn hash_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    hex::encode(hasher.finalize())
}
