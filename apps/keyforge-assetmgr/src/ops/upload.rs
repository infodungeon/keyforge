// apps/keyforge-assetmgr/src/ops/upload.rs

use crate::is_hidden;
use anyhow::Result;
use keyforge_infra::net::distributed::DistributedCoordinator;
use keyforge_model::types::path::SafePath;
use keyforge_protocol::AssetManifestEntry;
use std::path::Path;
use tracing::info;

/// Uploads a single file to the distributed asset store.
///
/// # Errors
/// Returns an error if the file cannot be read, hashed, or uploaded to Valkey.
#[allow(clippy::panic, clippy::expect_used, clippy::missing_panics_doc)]
pub async fn upload_file(
    coordinator: &dyn DistributedCoordinator,
    root: &Path,
    path: &Path,
) -> Result<()> {
    let rel = path.strip_prefix(root)?;
    let key_path = rel.to_string_lossy().replace('\\', "/");
    let valkey_key = format!("asset:blob:{key_path}");

    if is_hidden(path) {
        return Ok(());
    }

    let content = tokio::fs::read(path).await?;
    let size = content.len() as u64;

    let safe_path = SafePath::from_trusted_root_path(path.to_path_buf());
    let hash = keyforge_infra::util::common::calculate_file_hash(&safe_path)
        .map_err(|e| anyhow::anyhow!(e))?;

    if let Ok(Some(remote_hash)) = coordinator.get_manifest_hash(&key_path).await {
        if remote_hash == hash {
            return Ok(());
        }
    }

    coordinator
        .set_bin(&valkey_key, &content)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    let entry = AssetManifestEntry {
        id: key_path.clone(),
        hash,
        size_bytes: size,
        #[allow(clippy::cast_sign_loss)]
        updated_at: chrono::Utc::now().timestamp() as u64,
    };
    coordinator
        .set_manifest_entry(&entry)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    info!("⬆️  Synced: {}", key_path);
    Ok(())
}
