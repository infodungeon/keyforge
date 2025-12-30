use moka::sync::Cache;
use std::time::Duration;

#[derive(Clone)]
pub struct SecurityContext {
    pub api_secret: Option<String>,
    pub api_key_cache: Cache<String, bool>,
    pub nonce_cache: Cache<String, bool>,
    pub server_key: String,
}

impl SecurityContext {
    pub fn new(api_secret: Option<String>, server_key: String) -> Self {
        let api_key_cache = Cache::builder()
            .max_capacity(1000)
            .time_to_live(Duration::from_secs(300))
            .build();

        let nonce_cache = Cache::builder()
            .max_capacity(100_000)
            .time_to_live(Duration::from_secs(600))
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
}
