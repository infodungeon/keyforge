// ===== keyforge/crates/keyforge-hive/src/state.rs =====
use crate::queue::WriteQueue;
use crate::store::Store;
use keyforge_core::keycodes::KeycodeRegistry;
use keyforge_core::verifier::Verifier;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct AppState {
    pub store: Store,
    pub queue: Arc<WriteQueue>,
    pub registry: Arc<KeycodeRegistry>,

    // In-Memory Verifier Cache
    pub verifiers: Arc<RwLock<HashMap<String, Verifier>>>,

    // Security
    pub api_secret: Option<String>,
}

impl AppState {
    pub fn new(db: Pool<Postgres>, registry: KeycodeRegistry) -> Self {
        let store = Store::new(db);
        let queue = Arc::new(WriteQueue::new(store.clone(), 10_000));

        // Load Secret from Env (Standard 12-Factor App pattern)
        let api_secret = env::var("HIVE_SECRET").ok().filter(|s| !s.is_empty());

        if api_secret.is_some() {
            tracing::info!("🔒 HIVE_SECRET loaded. Authentication enabled.");
        } else {
            tracing::warn!("⚠️  HIVE_SECRET not set. Server is running in INSECURE mode.");
        }

        Self {
            store,
            queue,
            registry: Arc::new(registry),
            verifiers: Arc::new(RwLock::new(HashMap::new())),
            api_secret,
        }
    }
}
