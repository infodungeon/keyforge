// libs/keyforge-infra/src/net/distributed.rs

use crate::error::{InfraError, InfraResult};
use fred::prelude::*;
use fred::types::config::Config as RedisConfig;
use fred::types::{Expiration, Builder};
use fred::types::scan::Scanner;
use fred::clients::Client;
use keyforge_protocol::{AssetManifestEntry, NodeTelemetry};
use std::time::Duration;
use tracing::{info};
use futures::stream::StreamExt;

#[derive(Clone)]
pub struct DistributedCoordinator {
    client: Client,
}

impl DistributedCoordinator {
    /// Establish a connection to the Valkey instance.
    pub async fn new(url: &str) -> InfraResult<Self> {
        // Fred v10: Config::from_url
        let config = RedisConfig::from_url(url).map_err(|e| {
            InfraError::Config(format!("Invalid Valkey URL: {}", e))
        })?;
        
        // Fred v10: Builder pattern
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
        
        // Fred v10: Explicit init
        client.init().await.map_err(|e| {
            InfraError::Io(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                format!("Valkey connection failed: {}", e)
            ))
        })?;
        
        info!("✅ Connected to Coordination Layer (Valkey)");
        Ok(Self { client })
    }

    /// Updates the heartbeat and telemetry for a node.
    pub async fn update_heartbeat(&self, node_id: &str, telemetry: &NodeTelemetry) -> InfraResult<()> {
        let key = format!("v4:node:{}:telemetry", node_id);
        
        let bytes = postcard::to_stdvec(telemetry).map_err(|e| {
            InfraError::Serde(serde::ser::Error::custom(e))
        })?;

        self.client.set::<(), _, _>(
            key, 
            bytes, 
            Some(Expiration::EX(30)), 
            None, 
            false
        ).await.map_err(|e| {
             InfraError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
        })?;

        Ok(())
    }

    /// Retrieves the active telemetry for a specific node.
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

    /// Aggregates statistics from all active nodes in the cluster.
    /// Returns (node_count, total_ops_per_sec).
    pub async fn get_cluster_stats(&self) -> InfraResult<(usize, f32)> {
        let mut stream = self.client.scan("v4:node:*:telemetry", Some(1000), None);
        let mut keys = Vec::new();
        
        while let Some(res) = stream.next().await {
            match res {
                Ok(mut page) => {
                    if let Some(k) = page.take_results() {
                        keys.extend(k);
                    }
                }
                Err(_) => break,
            }
        }

        if keys.is_empty() {
            return Ok((0, 0.0));
        }

        // MGET all telemetry keys
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

    /// Publishes a job update event to the cluster.
    pub async fn publish_update(&self, job_id: &str, event: &str) -> InfraResult<()> {
        let channel = format!("job:{}:updates", job_id);
        self.client.publish::<(), _, _>(channel, event).await.map_err(|e| {
             InfraError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
        })?;
        Ok(())
    }

    /// Sets the authoritative hash for a system asset in the distributed manifest.
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
        })?;
        
        Ok(())
    }

    /// Retrieves the authoritative hash for an asset.
    pub async fn get_manifest_hash(&self, asset_id: &str) -> InfraResult<Option<String>> {
        let key = format!("v4:manifest:{}", asset_id);
        let hash: Option<String> = self.client.hget(key, "hash").await.map_err(|e| {
             InfraError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
        })?;
        Ok(hash)
    }

    /// Counts the number of active nodes by scanning for telemetry keys.
    pub async fn count_active_nodes(&self) -> InfraResult<usize> {
        let (count, _) = self.get_cluster_stats().await?;
        Ok(count)
    }
}
