// ===== keyforge/crates/keyforge-hive/src/state.rs =====
use crate::queue::WriteQueue;
use crate::store::Store;
use keyforge_core::keycodes::KeycodeRegistry;
use sqlx::{Pool, Postgres}; // FIXED: Changed Sqlite to Postgres
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub store: Store,
    pub queue: Arc<WriteQueue>,
    #[allow(dead_code)]
    pub registry: Arc<KeycodeRegistry>,
}

impl AppState {
    // FIXED: Signature accepts Pool<Postgres>
    pub fn new(db: Pool<Postgres>, registry: KeycodeRegistry) -> Self {
        let store = Store::new(db);
        let queue = Arc::new(WriteQueue::new(store.clone(), 10_000));

        Self {
            store,
            queue,
            registry: Arc::new(registry),
        }
    }
}
