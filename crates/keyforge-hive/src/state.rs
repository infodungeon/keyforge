use crate::queue::WriteQueue;
use crate::store::Store;
use keyforge_core::keycodes::KeycodeRegistry;
use sqlx::{Pool, Sqlite};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub store: Store,
    pub queue: Arc<WriteQueue>,
    pub registry: Arc<KeycodeRegistry>,
}

impl AppState {
    pub fn new(db: Pool<Sqlite>, registry: KeycodeRegistry) -> Self {
        let store = Store::new(db);
        // Buffer 10,000 writes in memory
        let queue = Arc::new(WriteQueue::new(store.clone(), 10_000));

        Self {
            store,
            queue,
            registry: Arc::new(registry),
        }
    }
}
