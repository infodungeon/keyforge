// apps/keyforge-hive/src/cache.rs

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


use moka::sync::Cache;
use std::sync::Arc;
use std::time::Duration;

// Re-export CompiledEngineCache
/// An LRU cache for pre-hydrated `ScoringEngine` instances.
///
/// This avoids the overhead of reloading and parsing corpora/keyboards for 
/// every verification request on the same job.
#[derive(Debug)]
pub struct CompiledEngineCache {
    cache: Cache<String, Arc<keyforge_core::ScoringEngine>>,
}

impl Default for CompiledEngineCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Default capacity for the engine cache.
pub const DEFAULT_ENGINE_CACHE_CAPACITY: u64 = 500;
/// Default TTL for cached engines (30 minutes).
pub const DEFAULT_ENGINE_CACHE_TTL_SECS: u64 = 1800;

impl CompiledEngineCache {
    /// Creates a new `CompiledEngineCache` with default capacity and TTL.
    #[must_use] 
    pub fn new() -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(DEFAULT_ENGINE_CACHE_CAPACITY)
                .time_to_live(Duration::from_secs(DEFAULT_ENGINE_CACHE_TTL_SECS))
                .build(),
        }
    }

    /// Retrieves a cached engine by job ID.
    #[must_use] 
    pub fn get(&self, job_id: &str) -> Option<Arc<keyforge_core::ScoringEngine>> {
        self.cache.get(job_id)
    }

    /// Inserts a hydrated engine into the cache.
    pub fn insert(&self, job_id: &str, engine: Arc<keyforge_core::ScoringEngine>) {
        self.cache.insert(job_id.to_string(), engine);
    }

    /// Clears all entries from the cache.
    pub fn invalidate_all(&self) {
        self.cache.invalidate_all();
    }
}
