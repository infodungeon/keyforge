// libs/keyforge-infra/src/asset/manager.rs

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

use crate::error::InfraResult;
use crate::net::client::HiveClient;
use crate::net::network::ensure_file;
use keyforge_model::CostMatrixSource;
use keyforge_protocol::JobConfig;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tracing::info;

/// Service responsible for orchestrating the presence and integrity of assets on the local node.
///
/// It coordinates with the Hive API to download missing assets and verifies their
/// authenticity using cryptographic hashes.
pub struct AssetManager {
    client: HiveClient,
    root: PathBuf,
}

impl AssetManager {
    /// Creates a new `AssetManager` instance.
    pub fn new(client: HiveClient, root: PathBuf) -> Self {
        Self { client, root }
    }

    /// Ensures the specified keyboard definition is present locally.
    ///
    /// If missing, it will be downloaded from the Hive.
    pub async fn ensure_keyboard(&self, name: &str) -> InfraResult<PathBuf> {
        let safe_name = crate::util::common::sanitize_filename(name);
        let filename = format!("{}.json", safe_name);
        let local_path = self.root.join("keyboards").join(&filename);
        let remote_path = format!("data/keyboards/{}", filename);
        let url = self.client.url(&remote_path);

        ensure_file(&self.client, &url, &local_path, None).await?;
        Ok(local_path)
    }

    /// Ensures the specified cost matrix file is present locally.
    pub async fn ensure_cost_matrix(&self, filename: &str) -> InfraResult<PathBuf> {
        let local_path = self.root.join(filename);
        let remote_path = format!("data/{}", filename);
        let url = self.client.url(&remote_path);

        ensure_file(&self.client, &url, &local_path, None).await?;
        Ok(local_path)
    }

    /// Ensures a corpus bundle (set of JSON files) is present and optionally matches a hash.
    ///
    /// This handles multi-file downloads and integrity verification.
    pub async fn ensure_corpus(
        &self,
        corpus_id: &str,
        expected_hash: Option<&str>,
    ) -> InfraResult<PathBuf> {
        let bundle_name = if corpus_id.is_empty() || corpus_id == "default" {
            "text/en_std"
        } else {
            corpus_id
        };

        let bundle_dir = self.root.join("corpora").join(bundle_name);
        let files = ["1grams.json", "2grams.json", "3grams.json", "words.json"];

        // 1. Verify existing
        if let Some(hash) = expected_hash {
            let root = self.root.clone();
            let bundle_name_owned = bundle_name.to_string();
            let hash_owned = hash.to_string();

            let provider = crate::FsProvider::new(root);
            let match_found = match provider.get_corpus_hash(&bundle_name_owned).await {
                Ok(h) => h == hash_owned,
                Err(_) => false,
            };

            if match_found {
                return Ok(bundle_dir);
            }

            // Mismatch: Clean up to force download
            info!(
                "Corpus hash mismatch for '{}'. Re-downloading.",
                bundle_name
            );
            for f in files {
                let p = bundle_dir.join(f);
                if p.exists() {
                    let _ = tokio::fs::remove_file(p).await;
                }
            }
        }

        for f in files {
            let local_path = bundle_dir.join(f);
            let remote_path = format!("data/corpora/{}/{}", bundle_name, f);
            let url = self.client.url(&remote_path);

            ensure_file(&self.client, &url, &local_path, None).await?;
        }

        // 3. Verify after download (if hash provided)
        if let Some(hash) = expected_hash {
            let root = self.root.clone();
            let bundle_name_owned = bundle_name.to_string();
            let hash_owned = hash.to_string();

            let provider = crate::FsProvider::new(root);
            let match_found = match provider.get_corpus_hash(&bundle_name_owned).await {
                Ok(h) => h == hash_owned,
                Err(_) => false,
            };

            if !match_found {
                return Err(crate::error::InfraError::HashMismatch {
                    expected: hash.to_string(),
                    actual: "mismatch after download".to_string(),
                });
            }
        }

        Ok(bundle_dir)
    }

    /// Syncs all assets required for a specific job configuration.
    ///
    /// Returns a tuple of `(cost_matrix_filename, corpora_directory_name)`.
    pub async fn sync_job_assets(&self, config: &JobConfig) -> InfraResult<(String, String)> {
        info!("📦 Syncing assets for job...");

        let cost_path = match &config.cost_matrix {
            CostMatrixSource::Predefined(filename) => {
                self.ensure_cost_matrix(filename).await?;
                filename.clone()
            }
            CostMatrixSource::Custom(content) => {
                let mut hasher = Sha256::new();
                hasher.update(content);
                let hash = hex::encode(hasher.finalize());
                let filename = format!("custom_cost_{}.json", hash);
                let path = self.root.join(&filename);
                if !path.exists() {
                    crate::fs::io::atomic_write(&path, content)?;
                }
                filename
            }
        };

        for source in &config.corpora {
            self.ensure_corpus(&source.id, source.hash.as_deref())
                .await?;
        }

        Ok((cost_path, "corpora".to_string()))
    }
}
