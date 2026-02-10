// libs/keyforge-infra/src/net/local_coord.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You    may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::error::InfraResult;
use crate::net::distributed::DistributedCoordinator;
use async_trait::async_trait;
use keyforge_protocol::{AssetManifestEntry, NodeTelemetry};
use std::collections::HashMap;
use tokio::sync::RwLock;

/// A non-distributed implementation of the `DistributedCoordinator` trait.
/// 
/// This is used when KeyForge Hive is running in single-node mode without 
/// an external Valkey/Redis instance. All state is kept in memory.
#[derive(Debug, Default)]
pub struct LocalDistributedCoordinator {
    bin_store: RwLock<HashMap<String, bytes::Bytes>>,
    heartbeats: RwLock<HashMap<String, NodeTelemetry>>,
    manifest: RwLock<HashMap<String, String>>,
    nonces: RwLock<HashMap<String, Vec<u64>>>,
}

impl LocalDistributedCoordinator {
    /// Creates a new `LocalDistributedCoordinator`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl DistributedCoordinator for LocalDistributedCoordinator {
    async fn get_bin(&self, key: &str) -> InfraResult<Option<bytes::Bytes>> {
        Ok(self.bin_store.read().await.get(key).cloned())
    }

    async fn set_bin(&self, key: &str, data: &[u8]) -> InfraResult<()> {
        self.bin_store.write().await.insert(key.to_string(), bytes::Bytes::copy_from_slice(data));
        Ok(())
    }

    async fn scan_keys(&self, pattern: &str) -> InfraResult<Vec<String>> {
        let store = self.bin_store.read().await;
        // Simple glob-to-contains mapping for local simulation
        let clean_pattern = pattern.replace('*', "");
        Ok(store.keys()
            .filter(|k| k.contains(&clean_pattern))
            .cloned()
            .collect())
    }

    async fn try_reserve_profile_update(&self, _cpu_signature: &str) -> InfraResult<bool> {
        // In local mode, always allow calibration updates
        Ok(true)
    }

    async fn update_heartbeat(&self, node_id: &str, telemetry: &NodeTelemetry) -> InfraResult<()> {
        self.heartbeats.write().await.insert(node_id.to_string(), telemetry.clone());
        Ok(())
    }

    async fn get_heartbeat(&self, node_id: &str) -> InfraResult<Option<NodeTelemetry>> {
        Ok(self.heartbeats.read().await.get(node_id).cloned())
    }

    async fn get_cluster_stats(&self) -> InfraResult<(usize, f32)> {
        let heartbeats = self.heartbeats.read().await;
        let count = heartbeats.len();
        let total_ips = heartbeats.values().map(|t| t.ips).sum();
        Ok((count, total_ips))
    }

    async fn publish_update(&self, _job_id: &str, _event: &str) -> InfraResult<()> {
        // No-op in local mode (no pub/sub listeners)
        Ok(())
    }

    async fn set_manifest_entry(&self, entry: &AssetManifestEntry) -> InfraResult<()> {
        self.manifest.write().await.insert(entry.id.clone(), entry.hash.clone());
        Ok(())
    }

    async fn get_manifest_hash(&self, asset_id: &str) -> InfraResult<Option<String>> {
        Ok(self.manifest.read().await.get(asset_id).cloned())
    }

    async fn get_all_manifest_entries(&self) -> InfraResult<HashMap<String, String>> {
        Ok(self.manifest.read().await.clone())
    }

    async fn count_active_nodes(&self) -> InfraResult<usize> {
        Ok(self.heartbeats.read().await.len())
    }

    async fn check_and_set_nonce(
        &self,
        node_id: &str,
        nonce: u64,
        _ttl_secs: i64,
    ) -> InfraResult<bool> {
        let mut nonces = self.nonces.write().await;
        let entry = nonces.entry(node_id.to_string()).or_default();
        if entry.contains(&nonce) {
            Ok(false)
        } else {
            entry.push(nonce);
            Ok(true)
        }
    }

    async fn release_profile_update(&self, _cpu_signature: &str) -> InfraResult<()> {
        // No-op in local mode as try_reserve_profile_update always returns true
        Ok(())
    }
}
