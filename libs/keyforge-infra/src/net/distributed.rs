// libs/keyforge-infra/src/net/distributed.rs

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

use crate::error::{InfraError, InfraResult};
use fred::clients::Client;
use fred::prelude::*;
use fred::types::config::Config as RedisConfig;
use fred::types::scan::Scanner;
use fred::types::{Builder, Expiration, SetOptions};
use futures::stream::StreamExt;
use keyforge_model::constants::{
    DEFAULT_CONNECT_TIMEOUT_SECS, DISTRIBUTED_KEY_VERSION, HEARTBEAT_TTL_SECS,
    PROFILE_LOCK_TTL_SECS,
};
use keyforge_protocol::{AssetManifestEntry, NodeTelemetry};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{debug, info};

// --- CONSTANTS ---

const KEY_PREFIX_V4: &str = DISTRIBUTED_KEY_VERSION;
const CONNECT_TIMEOUT_SEC: u64 = DEFAULT_CONNECT_TIMEOUT_SECS;
const PROFILE_LOCK_TTL_SEC: i64 = PROFILE_LOCK_TTL_SECS;
const HEARTBEAT_TTL_SEC: i64 = HEARTBEAT_TTL_SECS;

/// A coordinator that manages distributed state and communication across a cluster of nodes.
#[async_trait::async_trait]
pub trait DistributedCoordinator: Send + Sync + std::fmt::Debug {
    /// Retrieves binary data from the store by key.
    async fn get_bin(&self, key: &str) -> InfraResult<Option<bytes::Bytes>>;

    /// Stores binary data in the store with the specified key.
    async fn set_bin(&self, key: &str, data: &[u8]) -> InfraResult<()>;

    /// Scans for keys matching the given glob-style pattern.
    async fn scan_keys(&self, pattern: &str) -> InfraResult<Vec<String>>;

    /// Attempts to reserve an update slot for a hardware profile.
    async fn try_reserve_profile_update(&self, cpu_signature: &str) -> InfraResult<bool>;

    /// Updates the heartbeat and telemetry for a node.
    async fn update_heartbeat(&self, node_id: &str, telemetry: &NodeTelemetry) -> InfraResult<()>;

    /// Retrieves the latest telemetry for a specific node.
    async fn get_heartbeat(&self, node_id: &str) -> InfraResult<Option<NodeTelemetry>>;

    /// Aggregates statistics across all heartbeating nodes in the cluster.
    async fn get_cluster_stats(&self) -> InfraResult<(usize, f32)>;

    /// Publishes a job update event to a dedicated Pub/Sub channel.
    async fn publish_update(&self, job_id: &str, event: &str) -> InfraResult<()>;

    /// Sets an entry in the distributed asset manifest.
    async fn set_manifest_entry(&self, entry: &AssetManifestEntry) -> InfraResult<()>;

    /// Retrieves the hash of a specific asset from the distributed manifest.
    async fn get_manifest_hash(&self, asset_id: &str) -> InfraResult<Option<String>>;

    /// Fetches the entire distributed asset manifest as a map of ID to SHA-256 hash.
    async fn get_all_manifest_entries(&self) -> InfraResult<HashMap<String, String>>;

    /// Returns the number of currently active nodes in the cluster.
    async fn count_active_nodes(&self) -> InfraResult<usize>;

    /// Checks if a nonce has already been used by a node, and sets it if not.
    async fn check_and_set_nonce(
        &self,
        node_id: &str,
        nonce: u64,
        ttl_secs: i64,
    ) -> InfraResult<bool>;
}

/// A coordinator that manages distributed state and communication across a cluster of nodes.
///
/// It uses a central data store (Valkey/Redis) to handle heartbeats, cluster telemetry,
/// asset manifestation, and inter-node event publishing.
#[derive(Clone, Debug)]
pub struct ValkeyDistributedCoordinator {
    client: Client,
}

impl ValkeyDistributedCoordinator {
    /// Connects to the coordination layer using the provided Valkey/Redis URL.
    ///
    /// # Errors
    ///
    /// Returns `InfraError` if the URL is invalid or the connection fails.
    pub async fn new(url: &str) -> InfraResult<Self> {
        let config = RedisConfig::from_url(url)
            .map_err(|e| InfraError::Config(format!("Invalid Valkey URL: {e}")))?;

        let client = Builder::from_config(config)
            .with_connection_config(|c| {
                c.connection_timeout = Duration::from_secs(CONNECT_TIMEOUT_SEC);
            })
            .build()
            .map_err(|e| {
                InfraError::Io(std::io::Error::other(format!(
                    "Failed to build Valkey client: {e}"
                )))
            })?;

        client.init().await.map_err(|e| {
            InfraError::Io(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                format!("Valkey connection failed: {e}"),
            ))
        })?;

        info!("✅ Connected to Coordination Layer (Valkey)");
        Ok(Self { client })
    }
}

#[async_trait::async_trait]
impl DistributedCoordinator for ValkeyDistributedCoordinator {
    // --- BLOB STORAGE ---

    /// Retrieves binary data from the store by key.
    ///
    /// # Errors
    ///
    /// Returns `InfraError` if the underlying storage operation fails.
    async fn get_bin(&self, key: &str) -> InfraResult<Option<bytes::Bytes>> {
        self.client
            .get(key)
            .await
            .map_err(|e| InfraError::Io(std::io::Error::other(e)))
    }

    /// Stores binary data in the store with the specified key.
    ///
    /// # Errors
    ///
    /// Returns `InfraError` if the underlying storage operation fails.
    async fn set_bin(&self, key: &str, data: &[u8]) -> InfraResult<()> {
        self.client
            .set(key, data, None, None, false)
            .await
            .map_err(|e| InfraError::Io(std::io::Error::other(e)))
    }

    /// Scans for keys matching the given glob-style pattern.
    ///
    /// # Errors
    ///
    /// Returns `InfraError` if the underlying storage operation fails.
    async fn scan_keys(&self, pattern: &str) -> InfraResult<Vec<String>> {
        let mut stream = self.client.scan(pattern, Some(1000), None);
        let mut results = Vec::new();

        while let Some(res) = stream.next().await {
            match res {
                Ok(mut page) => {
                    if let Some(keys) = page.take_results() {
                        let strings: Vec<String> = keys
                            .into_iter()
                            .filter_map(|k| k.as_str().map(std::string::ToString::to_string))
                            .collect();
                        results.extend(strings);
                    }
                }
                Err(_) => break,
            }
        }
        Ok(results)
    }

    // --- COORDINATION ---

    /// Attempts to reserve an update slot for a hardware profile.
    ///
    /// This uses an atomic SET NX with a 24-hour expiration to ensure that
    /// calibration only happens once per day per hardware signature in a cluster.
    ///
    /// # Errors
    ///
    /// Returns `InfraError` if the underlying storage operation fails.
    async fn try_reserve_profile_update(&self, cpu_signature: &str) -> InfraResult<bool> {
        let key = format!("{KEY_PREFIX_V4}:hw_profile:{cpu_signature}");
        let result: Option<()> = self
            .client
            .set(
                key,
                "1",
                Some(Expiration::EX(PROFILE_LOCK_TTL_SEC)),
                Some(SetOptions::NX),
                false,
            )
            .await
            .map_err(|e| InfraError::Io(std::io::Error::other(e)))?;

        let is_new = result.is_some();
        if is_new {
            debug!("🆕 New Hardware Profile detected: {}", cpu_signature);
        }
        Ok(is_new)
    }

    /// Updates the heartbeat and telemetry for a node.
    ///
    /// The entry will automatically expire if not refreshed within 30 seconds,
    /// indicating that the node is offline.
    ///
    /// # Errors
    ///
    /// Returns `InfraError` if the underlying storage operation fails.
    async fn update_heartbeat(&self, node_id: &str, telemetry: &NodeTelemetry) -> InfraResult<()> {
        let key = format!("{KEY_PREFIX_V4}:node:{node_id}:telemetry");
        let json = serde_json::to_string(telemetry).map_err(InfraError::Serde)?;

        // 1. Update Detail Key (TTL-based)
        self.client
            .set::<(), _, _>(
                key,
                json,
                Some(Expiration::EX(HEARTBEAT_TTL_SEC)),
                None,
                false,
            )
            .await
            .map_err(|e| InfraError::Io(std::io::Error::other(e)))?;

        // 2. Update Active Set (Task-infra-016: O(1) counting)
        let zset_key = format!("{KEY_PREFIX_V4}:cluster:active_nodes");
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        #[allow(clippy::cast_precision_loss)]
        let score = now as f64;
        self.client
            .zadd::<(), _, _>(
                zset_key,
                None,
                None,
                false,
                false,
                (score, node_id.to_string()),
            )
            .await
            .map_err(|e| InfraError::Io(std::io::Error::other(e)))?;

        Ok(())
    }

    /// Retrieves the latest telemetry for a specific node.
    ///
    /// # Errors
    ///
    /// Returns `InfraError` if the underlying storage operation or parsing fails.
    async fn get_heartbeat(&self, node_id: &str) -> InfraResult<Option<NodeTelemetry>> {
        let key = format!("{KEY_PREFIX_V4}:node:{node_id}:telemetry");
        let data: Option<String> = self
            .client
            .get(key)
            .await
            .map_err(|e| InfraError::Io(std::io::Error::other(e)))?;
        if let Some(s) = data {
            let t = serde_json::from_str(&s).map_err(InfraError::Serde)?;
            Ok(Some(t))
        } else {
            Ok(None)
        }
    }

    /// Aggregates statistics across all heartbeating nodes in the cluster.
    ///
    /// Returns a tuple of `(active_node_count, aggregate_throughput_ips)`.
    ///
    /// # Errors
    ///
    /// Returns `InfraError` if the underlying storage operation fails.
    async fn get_cluster_stats(&self) -> InfraResult<(usize, f32)> {
        let zset_key = format!("{KEY_PREFIX_V4}:cluster:active_nodes");
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let threshold = now.saturating_sub(HEARTBEAT_TTL_SEC as u64);

        // 1. Prune stale nodes from ZSET
        self.client
            .zremrangebyscore::<(), _, _, _>(zset_key.clone(), "-inf", threshold.to_string())
            .await
            .ok(); // Ignore errors in pruning, next call will catch it

        // 2. O(1) Count
        let count: usize = self
            .client
            .zcard(zset_key.clone())
            .await
            .map_err(|e| InfraError::Io(std::io::Error::other(e)))?;

        if count == 0 {
            return Ok((0, 0.0));
        }

        // 3. Throughput
        let node_ids: Vec<String> = self
            .client
            .zrange(zset_key, 0, -1, None, false, None, false)
            .await
            .map_err(|e| InfraError::Io(std::io::Error::other(e)))?;

        if node_ids.is_empty() {
            return Ok((0, 0.0));
        }

        let keys: Vec<String> = node_ids
            .into_iter()
            .map(|id| format!("{KEY_PREFIX_V4}:node:{id}:telemetry"))
            .collect();

        let values: Vec<Option<String>> = self
            .client
            .mget(keys)
            .await
            .map_err(|e| InfraError::Io(std::io::Error::other(e)))?;

        let mut total_ops = 0.0;
        for val in values.into_iter().flatten() {
            if let Ok(telemetry) = serde_json::from_str::<NodeTelemetry>(&val) {
                total_ops += telemetry.ips;
            }
        }
        Ok((count, total_ops))
    }

    /// Publishes a job update event to a dedicated Pub/Sub channel.
    ///
    /// # Errors
    ///
    /// Returns `InfraError` if the underlying storage operation fails.
    async fn publish_update(&self, job_id: &str, event: &str) -> InfraResult<()> {
        let channel = format!("job:{job_id}:updates");
        self.client
            .publish::<(), _, _>(channel, event)
            .await
            .map_err(|e| InfraError::Io(std::io::Error::other(e)))
    }

    /// Sets an entry in the distributed asset manifest.
    ///
    /// # Errors
    ///
    /// Returns `InfraError` if the underlying storage operation fails.
    async fn set_manifest_entry(&self, entry: &AssetManifestEntry) -> InfraResult<()> {
        let key = format!("{}:manifest:{}", KEY_PREFIX_V4, entry.id);
        self.client
            .hset::<(), _, _>(
                key,
                vec![
                    ("hash", entry.hash.as_str()),
                    ("size", &entry.size_bytes.to_string()),
                    ("updated", &entry.last_updated.to_string()),
                ],
            )
            .await
            .map_err(|e| InfraError::Io(std::io::Error::other(e)))
    }

    /// Retrieves the hash of a specific asset from the distributed manifest.
    ///
    /// # Errors
    ///
    /// Returns `InfraError` if the underlying storage operation fails.
    async fn get_manifest_hash(&self, asset_id: &str) -> InfraResult<Option<String>> {
        let key = format!("{KEY_PREFIX_V4}:manifest:{asset_id}");
        let hash: Option<String> = self
            .client
            .hget(key, "hash")
            .await
            .map_err(|e| InfraError::Io(std::io::Error::other(e)))?;
        Ok(hash)
    }

    /// Fetches the entire distributed asset manifest as a map of ID to SHA-256 hash.
    ///
    /// # Errors
    ///
    /// Returns `InfraError` if the underlying storage operation fails.
    async fn get_all_manifest_entries(&self) -> InfraResult<HashMap<String, String>> {
        let keys = self
            .scan_keys(&format!("{KEY_PREFIX_V4}:manifest:*"))
            .await?;
        let mut map = HashMap::new();

        for key in keys {
            if let Some(id) = key.strip_prefix(&format!("{KEY_PREFIX_V4}:manifest:")) {
                let hash: Option<String> = self
                    .client
                    .hget(&key, "hash")
                    .await
                    .map_err(|e| InfraError::Io(std::io::Error::other(e)))?;

                if let Some(h) = hash {
                    map.insert(id.to_string(), h);
                }
            }
        }
        Ok(map)
    }

    /// Returns the number of currently active nodes in the cluster.
    ///
    /// # Errors
    ///
    /// Returns `InfraError` if the underlying storage operation fails.
    async fn count_active_nodes(&self) -> InfraResult<usize> {
        let (count, _) = self.get_cluster_stats().await?;
        Ok(count)
    }

    /// Checks if a nonce has already been used by a node, and sets it if not.
    ///
    /// This provides distributed replay protection with a specified TTL.
    /// Returns true if the nonce is NEW (not seen before), false if it's a REPLAY.
    ///
    /// # Errors
    ///
    /// Returns `InfraError` if the underlying storage operation fails.
    async fn check_and_set_nonce(
        &self,
        node_id: &str,
        nonce: u64,
        ttl_secs: i64,
    ) -> InfraResult<bool> {
        let key = format!("{KEY_PREFIX_V4}:nonce:{node_id}:{nonce}");
        let result: Option<()> = self
            .client
            .set(
                key,
                "1",
                Some(Expiration::EX(ttl_secs)),
                Some(SetOptions::NX),
                false,
            )
            .await
            .map_err(|e| InfraError::Io(std::io::Error::other(e)))?;

        Ok(result.is_some())
    }
}
