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
