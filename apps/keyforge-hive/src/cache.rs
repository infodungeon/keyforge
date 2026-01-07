// apps/keyforge-hive/src/cache.rs

use moka::sync::Cache;
use std::sync::Arc;
use std::time::Duration;

// Re-export CompiledEngineCache
pub struct CompiledEngineCache {
    cache: Cache<String, Arc<keyforge_core::ScoringEngine>>,
}

impl Default for CompiledEngineCache {
    fn default() -> Self {
        Self::new()
    }
}

impl CompiledEngineCache {
    pub fn new() -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(500)
                .time_to_live(Duration::from_secs(1800))
                .build(),
        }
    }

    pub fn get(&self, job_id: &str) -> Option<Arc<keyforge_core::ScoringEngine>> {
        self.cache.get(job_id)
    }

    pub fn insert(&self, job_id: &str, engine: Arc<keyforge_core::ScoringEngine>) {
        self.cache.insert(job_id.to_string(), engine);
    }

    pub fn invalidate_all(&self) {
        self.cache.invalidate_all();
    }
}
