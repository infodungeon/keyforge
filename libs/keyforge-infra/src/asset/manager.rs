// libs/keyforge-infra/src/asset/manager.rs

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
use crate::net::client::HiveClient;
use keyforge_model::types::path::SafePath;
use keyforge_protocol::JobConfig;
use std::path::PathBuf;
use tracing::info;

/// Orchestrates asset management, ensuring required files are present and synchronized.
///
/// `AssetManager` acts as a high-level controller that uses a `HiveClient` to interact with
/// the remote Hive service and manages a local directory of assets.
#[derive(Debug)]
pub struct AssetManager {
    client: HiveClient,
    root: SafePath,
}

impl AssetManager {
    /// Creates a new `AssetManager` with the given client and root directory.
    #[must_use]
    pub fn new(client: HiveClient, root: SafePath) -> Self {
        Self { client, root }
    }

    /// Returns the root directory managed by this `AssetManager`.
    #[must_use]
    pub fn root(&self) -> &SafePath {
        &self.root
    }

    /// Synchronizes the local asset library with the remote Hive.
    ///
    /// # Errors
    ///
    /// Returns an error if the network request or file I/O fails.
    pub async fn sync(&self) -> InfraResult<()> {
        info!("🔄 Synchronizing assets with Hive...");
        crate::net::sync::run_sync(&self.client, &self.root).await?;
        info!("✅ Asset synchronization complete.");
        Ok(())
    }

    /// Ensures all assets required for a specific job are present locally.
    ///
    /// # Errors
    ///
    /// Returns an error if any asset cannot be retrieved.
    pub async fn sync_job_assets(&self, config: &JobConfig) -> InfraResult<()> {
        let _ = self.ensure_keyboard(&config.definition.meta.name).await?;
        for corpus in &config.corpora {
            self.ensure_corpus(&corpus.id, corpus.hash.as_deref())
                .await?;
        }
        // Additional assets like custom cost matrices could be handled here.
        Ok(())
    }

    /// Ensures a specific keyboard definition is present locally, downloading it if necessary.
    /// Returns the local path to the asset.
    ///
    /// # Errors
    ///
    /// Returns an error if the asset cannot be retrieved or saved.
    pub async fn ensure_keyboard(&self, id: &str) -> InfraResult<PathBuf> {
        let path = self
            .root
            .as_path()
            .join("system/keyboards")
            .join(format!("{id}.json"));
        if !path.exists() {
            info!("📥 Downloading missing keyboard: {}", id);
            self.client.download_asset("keyboards", id, &path).await?;
        }
        Ok(path)
    }

    /// Ensures a specific corpus is present locally.
    ///
    /// # Errors
    ///
    /// Returns an error if the corpus cannot be retrieved.
    pub async fn ensure_corpus(&self, id: &str, _hash: Option<&str>) -> InfraResult<()> {
        let path = self.root.as_path().join("system/corpora").join(id);
        if !path.exists() {
            info!("📥 Downloading missing corpus: {}", id);
            self.client.download_asset("corpora", id, &path).await?;
        }
        Ok(())
    }

    /// Ensures a specific cost matrix is present locally.
    ///
    /// # Errors
    ///
    /// Returns an error if the cost matrix cannot be retrieved.
    pub async fn ensure_cost_matrix(&self, id: &str) -> InfraResult<()> {
        let path = self
            .root
            .as_path()
            .join("system/cost_matrices")
            .join(format!("{id}.json"));
        if !path.exists() {
            info!("📥 Downloading missing cost matrix: {}", id);
            self.client
                .download_asset("cost_matrices", id, &path)
                .await?;
        }
        Ok(())
    }
}
