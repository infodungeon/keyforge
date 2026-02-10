use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A wrapper for loaded assets that includes a content-addressable hash.
#[derive(Debug, Serialize, Deserialize)]
pub struct Asset<T> {
    /// The actual asset data.
    pub content: Arc<T>,
    /// The SHA-256 hash of the raw asset content.
    pub content_hash: [u8; 32],
}

impl<T> Clone for Asset<T> {
    fn clone(&self) -> Self {
        Self {
            content: self.content.clone(),
            content_hash: self.content_hash,
        }
    }
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
