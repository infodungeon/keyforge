// libs/keyforge-infra/src/assets.rs

use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A wrapper for loaded assets that includes a content-addressable hash.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset<T> {
    /// The actual asset data.
    pub content: Arc<T>,
    /// The SHA-256 hash of the raw asset content.
    pub content_hash: [u8; 32],
}

impl<T> Asset<T> {
    /// Creates a new `Asset` wrapper.
    pub fn new(content: Arc<T>, hash: [u8; 32]) -> Self {
        Self {
            content,
            content_hash: hash,
        }
    }
}
