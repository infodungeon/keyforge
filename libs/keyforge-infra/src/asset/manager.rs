// libs/keyforge-infra/src/asset/manager.rs

use crate::error::InfraResult;
use crate::net::client::HiveClient;
use crate::net::network::ensure_file;
use keyforge_model::constants::{
    ASSET_1GRAMS_FILENAME, ASSET_2GRAMS_FILENAME, ASSET_3GRAMS_FILENAME, ASSET_WORDS_FILENAME,
    DEFAULT_CORPUS_ID,
};
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
    #[must_use] 
    pub fn new(client: HiveClient, root: PathBuf) -> Self {
        Self { client, root }
    }

    fn check_system_path(&self, category: &str, stem: &str) -> Option<PathBuf> {
        let sub = match category {
            "keyboards" => "keyboards/models",
            "weights" => "weights",
            "config" => "config",
            _ => category,
        };
        let p = self.root.join("system").join(sub).join(format!("{stem}.mpk.zst"));
        if p.exists() { Some(p) } else { None }
    }

    fn check_user_path(&self, category: &str, stem: &str) -> Option<PathBuf> {
        let p = self.root.join("user").join(category).join(format!("{stem}.json"));
        if p.exists() { Some(p) } else { None }
    }

    pub async fn ensure_keyboard(&self, name: &str) -> InfraResult<PathBuf> {
        let stem = name.strip_suffix(".json").unwrap_or(name);
        
        if let Some(p) = self.check_user_path("keyboards", stem) { return Ok(p); }
        if let Some(p) = self.check_system_path("keyboards", stem) { return Ok(p); }
        
        let local_path = self.root.join("user/keyboards").join(format!("{stem}.json"));
        let remote_path = format!("data/keyboards/{stem}.json");
        let url = self.client.asset_url(&remote_path);

        ensure_file(&self.client, &url, &local_path, None).await?;
        Ok(local_path)
    }

    pub async fn ensure_cost_matrix(&self, filename: &str) -> InfraResult<PathBuf> {
        let stem = filename.strip_suffix(".json").unwrap_or(filename);

        if let Some(p) = self.check_user_path("weights", stem) { return Ok(p); }
        if let Some(p) = self.check_system_path("weights", stem) { return Ok(p); }
        
        let local_path = self.root.join("user/weights").join(format!("{stem}.json"));
        let remote_path = format!("data/{stem}.json");
        let url = self.client.asset_url(&remote_path);

        ensure_file(&self.client, &url, &local_path, None).await?;
        Ok(local_path)
    }

    pub async fn ensure_corpus(
        &self,
        corpus_id: &str,
        expected_hash: Option<&str>,
    ) -> InfraResult<PathBuf> {
        let bundle_id = if corpus_id.is_empty() || corpus_id == "default" {
            DEFAULT_CORPUS_ID
        } else {
            corpus_id
        };

        // 1. Check System (Binary)
        let sys_dir = self.root.join("system/corpora").join(bundle_id);
        if sys_dir.join("1grams.mpk.zst").exists() {
            if let Some(hash) = expected_hash {
                let provider = crate::FsProvider::new(self.root.clone());
                if let Ok(h) = provider.get_corpus_hash(bundle_id).await {
                    if h == hash {
                        return Ok(sys_dir);
                    }
                    info!("System corpus '{}' hash mismatch. Falling back to User/Remote.", bundle_id);
                }
            } else {
                return Ok(sys_dir);
            }
        }

        // 2. Check User (JSON)
        let user_dir = self.root.join("user/corpora").join(bundle_id);
        let files = [
            ASSET_1GRAMS_FILENAME,
            ASSET_2GRAMS_FILENAME,
            ASSET_3GRAMS_FILENAME,
            ASSET_WORDS_FILENAME,
        ];

        let all_user_exist = files.iter().all(|f| user_dir.join(f).exists());
        
        if all_user_exist {
            if let Some(hash) = expected_hash {
                let provider = crate::FsProvider::new(self.root.clone());
                if let Ok(h) = provider.get_corpus_hash(bundle_id).await {
                    if h == hash {
                        return Ok(user_dir);
                    }
                }
            } else {
                return Ok(user_dir);
            }
        }

        // Download missing or mismatched files to USER directory
        for f in files {
            let local_path = user_dir.join(f);
            let remote_path = format!("data/corpora/{bundle_id}/{f}");
            let url = self.client.asset_url(&remote_path);

            ensure_file(&self.client, &url, &local_path, None).await?;
        }

        Ok(user_dir)
    }

    pub async fn sync_job_assets(&self, config: &JobConfig) -> InfraResult<(String, String)> {
        info!("📦 Syncing assets for job...");
        let cost_path = match &config.cost_matrix {
            CostMatrixSource::Predefined(filename) => {
                let p = self.ensure_cost_matrix(filename).await?;
                p.file_name().unwrap().to_string_lossy().to_string()
            }
        };
        for source in &config.corpora {
            self.ensure_corpus(&source.id, source.hash.as_deref()).await?;
        }
        Ok((cost_path, "corpora".to_string()))
    }
}
