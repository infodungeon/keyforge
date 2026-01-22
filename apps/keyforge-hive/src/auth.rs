// apps/keyforge-hive/src/auth.rs

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

use crate::state::AppState;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use keyforge_security as crypto;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use subtle::ConstantTimeEq;
use tracing::warn;

pub async fn require_secret(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // FAIL CLOSED: If no secret is configured, deny all requests.
    if state.security.api_secret.is_none() {
        warn!("⛔ Auth Failed: Server has no secret configured.");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // 1. Try PASETO Bearer Token (Task-sec-022)
    if let Some(auth_header) = req.headers().get(axum::http::header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                let key = state.security.get_token_key();
                if crypto::verify_paseto_token(&key, token).is_ok() {
                    return Ok(next.run(req).await);
                }
                warn!("⛔ Auth Failed: Invalid Bearer Token");
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
    }

    // 2. Fallback to Legacy X-Keyforge-Secret Header
    let auth_header = req
        .headers()
        .get("X-Keyforge-Secret")
        .and_then(|h| h.to_str().ok());

    let Some(token) = auth_header else {
        warn!("⛔ Auth Failed: Missing Authentication from {:?}", req.uri());
        return Err(StatusCode::UNAUTHORIZED);
    };

    // Check Master Key (HIVE_SECRET)
    if let Some(master) = &state.security.api_secret {
        if token.as_bytes().ct_eq(master.as_bytes()).into() {
            return Ok(next.run(req).await);
        }
    }

    // Check Database API Keys
    let hash = hash_key(token);
    if let Some(valid) = state.security.api_key_cache.get(&hash) {
        if valid {
            return Ok(next.run(req).await);
        }
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Use Repository
    let valid = state.users.validate_key(&hash).await.map_err(|e| {
        tracing::error!("Auth DB Error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Cache result (positive or negative)
    state.security.api_key_cache.insert(hash, valid);

    if valid {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_key() {
        let key = "test-key";
        let h1 = hash_key(key);
        let h2 = hash_key(key);
        assert_eq!(h1, h2);
        assert_ne!(h1, key);
    }
}
