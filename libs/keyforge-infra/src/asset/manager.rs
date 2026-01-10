// libs/keyforge-infra/src/asset/manager.rs

use crate::error::InfraResult;
use crate::net::client::HiveClient;
use crate::net::network::ensure_file;
use keyforge_model::CostMatrixSource;
use keyforge_protocol::JobConfig;
use std::path::PathBuf;
use tracing::info;

#[derive(Debug)]
pub struct AssetManager {
    client: HiveClient,
    root: PathBuf,
}

impl AssetManager {
    pub fn new(client: HiveClient, root: PathBuf) -> Self {
        Self { client, root }
    }

    pub async fn ensure_keyboard(&self, name: &str) -> InfraResult<PathBuf> {
        let safe_name = crate::util::common::sanitize_filename(name);
        let filename = format!("{}.json", safe_name);
        let local_path = self.root.join("keyboards").join(&filename);
        // Using Asset URL
        let remote_path = format!("data/keyboards/{}", filename);
        let url = self.client.asset_url(&remote_path);

        ensure_file(&self.client, &url, &local_path, None).await?;
        Ok(local_path)
    }

    pub async fn ensure_cost_matrix(&self, filename: &str) -> InfraResult<PathBuf> {
        let local_path = self.root.join(filename);
        let remote_path = format!("data/{}", filename);
        let url = self.client.asset_url(&remote_path);

        ensure_file(&self.client, &url, &local_path, None).await?;
        Ok(local_path)
    }

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

        if let Some(hash) = expected_hash {
            let root = self.root.clone();
            let provider = crate::FsProvider::new(root);
            if let Ok(h) = provider.get_corpus_hash(bundle_name).await {
                if h == hash { return Ok(bundle_dir); }
            }
            // Mismatch or missing, download
        }

        for f in files {
            let local_path = bundle_dir.join(f);
            let remote_path = format!("data/corpora/{}/{}", bundle_name, f);
            let url = self.client.asset_url(&remote_path);

            ensure_file(&self.client, &url, &local_path, None).await?;
        }

        Ok(bundle_dir)
    }

    pub async fn sync_job_assets(&self, config: &JobConfig) -> InfraResult<(String, String)> {
        info!("📦 Syncing assets for job...");
        let cost_path = match &config.cost_matrix {
            CostMatrixSource::Predefined(filename) => {
                self.ensure_cost_matrix(filename).await?;
                filename.clone()
            }
        };
        for source in &config.corpora {
            self.ensure_corpus(&source.id, source.hash.as_deref()).await?;
        }
        Ok((cost_path, "corpora".to_string()))
    }
}
