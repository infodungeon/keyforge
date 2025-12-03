use crate::store::Store;
use keyforge_core::keycodes::KeycodeRegistry;
use sqlx::{Pool, Sqlite};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub store: Store, // CHANGED: Replaced 'db' with 'store'
    pub registry: Arc<KeycodeRegistry>,
}

impl AppState {
    pub fn new(db: Pool<Sqlite>, registry: KeycodeRegistry) -> Self {
        Self {
            store: Store::new(db),
            registry: Arc::new(registry),
        }
    }
}
