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
use fred::prelude::*;
use fred::types::config::Config as RedisConfig;
use fred::types::{Expiration, Builder, SetOptions};
use fred::types::scan::Scanner;
use fred::clients::Client;
use keyforge_protocol::{AssetManifestEntry, NodeTelemetry};
use std::time::Duration;
use tracing::{info, debug};
use futures::stream::StreamExt;
use std::collections::HashMap;

/// A coordinator that manages distributed state and communication across a cluster of nodes.
///
/// It uses a central data store (Valkey/Redis) to handle heartbeats, cluster telemetry,
/// asset manifestation, and inter-node event publishing.
#[derive(Clone)]
pub struct DistributedCoordinator {
    client: Client,
}

impl DistributedCoordinator {
    /// Connects to the coordination layer using the provided Valkey/Redis URL.
    pub async fn new(url: &str) -> InfraResult<Self> {
        let config = RedisConfig::from_url(url).map_err(|e| {
            InfraError::Config(format!("Invalid Valkey URL: {}", e))
        })?;
        
        let client = Builder::from_config(config)
            .with_connection_config(|c| {
                c.connection_timeout = Duration::from_secs(10);
            })
            .build()
            .map_err(|e| {
                InfraError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to build Valkey client: {}", e)
                ))
            })?;
        
        client.init().await.map_err(|e| {
            InfraError::Io(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                format!("Valkey connection failed: {}", e)
            ))
        })?;
        
        info!("✅ Connected to Coordination Layer (Valkey)");
        Ok(Self { client })
    }

    // --- BLOB STORAGE ---

    /// Retrieves binary data from the store by key.
    pub async fn get_bin(&self, key: &str) -> InfraResult<Option<bytes::Bytes>> {
        self.client.get(key).await.map_err(|e| {
             InfraError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
        })
    }

    /// Stores binary data in the store with the specified key.
    pub async fn set_bin(&self, key: &str, data: &[u8]) -> InfraResult<()> {
        self.client.set(key, data, None, None, false).await.map_err(|e| {
             InfraError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
        })
    }

    /// Scans for keys matching the given glob-style pattern.
    pub async fn scan_keys(&self, pattern: &str) -> InfraResult<Vec<String>> {
        let mut stream = self.client.scan(pattern, Some(1000), None);
        let mut results = Vec::new();
        
        while let Some(res) = stream.next().await {
            match res {
                Ok(mut page) => {
                    if let Some(keys) = page.take_results() {
                        let strings: Vec<String> = keys.into_iter()
                            .filter_map(|k| k.as_str().map(|s| s.to_string()))
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
    pub async fn try_reserve_profile_update(&self, cpu_signature: &str) -> InfraResult<bool> {
        let key = format!("v4:hw_profile:{}", cpu_signature);
        let result: Option<()> = self.client.set(
            key,
            "1",
            Some(Expiration::EX(86400)),
            Some(SetOptions::NX),
            false
        ).await.map_err(|e| {
             InfraError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
        })?;
        
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
    pub async fn update_heartbeat(&self, node_id: &str, telemetry: &NodeTelemetry) -> InfraResult<()> {
        let key = format!("v4:node:{}:telemetry", node_id);
        let bytes = postcard::to_stdvec(telemetry).map_err(|e| {
            InfraError::Serde(serde::ser::Error::custom(e))
        })?;
        self.client.set::<(), _, _>(key, bytes, Some(Expiration::EX(30)), None, false)
            .await.map_err(|e| InfraError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))
    }

    /// Retrieves the latest telemetry for a specific node.
    pub async fn get_heartbeat(&self, node_id: &str) -> InfraResult<Option<NodeTelemetry>> {
        let key = format!("v4:node:{}:telemetry", node_id);
        let bytes: Option<bytes::Bytes> = self.client.get(key).await.map_err(|e| {
             InfraError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
        })?;
        if let Some(b) = bytes {
            let t = postcard::from_bytes(&b).map_err(|e| {
                InfraError::Serde(serde::de::Error::custom(e))
            })?;
            Ok(Some(t))
        } else {
            Ok(None)
        }
    }

    /// Aggregates statistics across all heartbeating nodes in the cluster.
    ///
    /// Returns a tuple of `(active_node_count, aggregate_throughput_ips)`.
    pub async fn get_cluster_stats(&self) -> InfraResult<(usize, f32)> {
        let keys = self.scan_keys("v4:node:*:telemetry").await?;
        if keys.is_empty() { return Ok((0, 0.0)); }

        let values: Vec<Option<bytes::Bytes>> = self.client.mget(keys).await.map_err(|e| {
             InfraError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
        })?;
        
        let mut count = 0;
        let mut total_ops = 0.0;
        for val in values.into_iter().flatten() {
            if let Ok(telemetry) = postcard::from_bytes::<NodeTelemetry>(&val) {
                count += 1;
                total_ops += telemetry.ips;
            }
        }
        Ok((count, total_ops))
    }

    /// Publishes a job update event to a dedicated Pub/Sub channel.
    pub async fn publish_update(&self, job_id: &str, event: &str) -> InfraResult<()> {
        let channel = format!("job:{}:updates", job_id);
        self.client.publish::<(), _, _>(channel, event).await.map_err(|e| {
             InfraError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
        })
    }

    /// Sets an entry in the distributed asset manifest.
    pub async fn set_manifest_entry(&self, entry: &AssetManifestEntry) -> InfraResult<()> {
        let key = format!("v4:manifest:{}", entry.id);
        self.client.hset::<(), _, _>(
            key, 
            vec![
                ("hash", entry.hash.as_str()),
                ("size", &entry.size_bytes.to_string()),
                ("updated", &entry.last_updated.to_string())
            ]
        ).await.map_err(|e| {
             InfraError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
        })
    }

    /// Retrieves the hash of a specific asset from the distributed manifest.
    pub async fn get_manifest_hash(&self, asset_id: &str) -> InfraResult<Option<String>> {
        let key = format!("v4:manifest:{}", asset_id);
        let hash: Option<String> = self.client.hget(key, "hash").await.map_err(|e| {
             InfraError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
        })?;
        Ok(hash)
    }

    /// Fetches the entire distributed asset manifest as a map of ID to SHA-256 hash.
    pub async fn get_all_manifest_entries(&self) -> InfraResult<HashMap<String, String>> {
        let keys = self.scan_keys("v4:manifest:*").await?;
        let mut map = HashMap::new();

        for key in keys {
            if let Some(id) = key.strip_prefix("v4:manifest:") {
                let hash: Option<String> = self.client.hget(&key, "hash").await.map_err(|e| {
                    InfraError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
                })?;
                
                if let Some(h) = hash {
                    map.insert(id.to_string(), h);
                }
            }
        }
        Ok(map)
    }

    /// Returns the number of currently active nodes in the cluster.
    pub async fn count_active_nodes(&self) -> InfraResult<usize> {
        let (count, _) = self.get_cluster_stats().await?;
        Ok(count)
    }
}
