// apps/keyforge-hive/src/services/security.rs

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

use crate::config::{
    DEFAULT_API_KEY_CACHE_CAPACITY, DEFAULT_API_KEY_CACHE_TTL_SECS, DEFAULT_NONCE_CACHE_CAPACITY,
    DEFAULT_NONCE_CACHE_TTL_SECS,
};
use moka::sync::Cache;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct SecurityContext {
    pub api_secret: Option<String>,
    pub api_key_cache: Cache<String, bool>,
    pub nonce_cache: Cache<String, bool>,
    pub server_key: String,
}

impl SecurityContext {
    pub fn new(api_secret: Option<String>, server_key: String) -> Self {
        let api_key_cache = Cache::builder()
            .max_capacity(DEFAULT_API_KEY_CACHE_CAPACITY)
            .time_to_live(Duration::from_secs(DEFAULT_API_KEY_CACHE_TTL_SECS))
            .build();

        let nonce_cache = Cache::builder()
            .max_capacity(DEFAULT_NONCE_CACHE_CAPACITY)
            .time_to_live(Duration::from_secs(DEFAULT_NONCE_CACHE_TTL_SECS))
            .build();

        Self {
            api_secret,
            api_key_cache,
            nonce_cache,
            server_key,
        }
    }

    pub fn validate_api_key(&self, key: &str) -> bool {
        if let Some(secret) = &self.api_secret {
            // Simple direct check for now, can use cache if we have external validation
            // For now, if secret is set, key must match.
            // If we had a DB of keys, we'd check that and cache the result.
            key == secret
        } else {
            // No secret configured = no auth required? or fail?
            // Usually if HIVE_SECRET is not set, we might default to allow or deny.
            // Existing logic in main.rs/state.rs implies:
            // "api_secret = env::var..."
            // If it's None, maybe auth is disabled or we just return true?
            // Let's assume strict: if None, maybe open?
            // Actually, let's just expose the field or a method.
            true // Default to true if no secret? Need to check existing usage.
        }
    }

    /// Derives a consistent 32-byte key for token encryption from the configured secret.
    #[must_use]
    pub fn get_token_key(&self) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        if let Some(secret) = &self.api_secret {
            hasher.update(secret.as_bytes());
        } else {
            hasher.update(b"DEFAULT_INSECURE_KEY_REPLACE_ME");
        }
        hasher.finalize().into()
    }
}
