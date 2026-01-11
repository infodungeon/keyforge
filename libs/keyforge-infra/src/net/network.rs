// libs/keyforge-infra/src/net/network.rs

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
use crate::net::client::HiveClient;
use backoff::ExponentialBackoff;
use futures_util::StreamExt;
use keyforge_model::constants::{
    ASSET_1GRAMS_FILENAME, ASSET_2GRAMS_FILENAME, ASSET_3GRAMS_FILENAME, ASSET_WORDS_FILENAME,
    DEFAULT_CORPUS_ID, MAX_INPUT_FILE_SIZE,
};
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::NamedTempFile;
use tracing::{info, warn};

/// Downloads a file securely with exponential backoff and streaming hash verification.
pub async fn ensure_file(
    client: &HiveClient,
    url: &str,
    local_path: &Path,
    expected_hash: Option<&str>,
) -> InfraResult<()> {
    // 1. Check existing file integrity
    if local_path.exists() {
        if let Some(hash) = expected_hash {
            // For existing files, we still have to read them to verify.
            // Optimization: We could store a .sha256 sidecar file to avoid re-hashing large files on every startup.
            // For now, we read.
            let content = tokio::fs::read(local_path).await.map_err(InfraError::Io)?;
            let mut hasher = Sha256::new();
            hasher.update(&content);
            let calculated = hex::encode(hasher.finalize());

            if calculated == hash {
                return Ok(());
            }
            warn!(
                "⚠️ Hash mismatch for {:?}. Expected {}, got {}. Re-downloading.",
                local_path, hash, calculated
            );
        } else {
            return Ok(());
        }
    }

    if let Some(parent) = local_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(InfraError::Io)?;
    }

    info!("⬇️ Downloading: {}", url);

    let op = || async {
        let res = client
            .get(url)
            .send()
            .await
            .map_err(|e| backoff::Error::transient(InfraError::Network(e)))?;

        if !res.status().is_success() {
            let status = res.status();
            let err = res.error_for_status().unwrap_err();
            if status.is_server_error() {
                return Err(backoff::Error::transient(InfraError::Network(err)));
            } else {
                return Err(backoff::Error::permanent(InfraError::Network(err)));
            }
        }

        // Security: Check Content-Length
        if let Some(len_header) = res.headers().get(reqwest::header::CONTENT_LENGTH) {
            if let Ok(len_str) = len_header.to_str() {
                if let Ok(len) = len_str.parse::<u64>() {
                    if len > MAX_INPUT_FILE_SIZE {
                        return Err(backoff::Error::permanent(InfraError::Io(
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!(
                                    "Remote file exceeds size limit ({} > {})",
                                    len, MAX_INPUT_FILE_SIZE
                                ),
                            ),
                        )));
                    }
                }
            }
        }

        Ok(res)
    };

    let backoff_conf = ExponentialBackoff {
        initial_interval: Duration::from_millis(500),
        randomization_factor: 0.5,
        multiplier: 1.5,
        max_interval: Duration::from_secs(10),
        max_elapsed_time: Some(Duration::from_secs(60)),
        ..Default::default()
    };

    let res = backoff::future::retry(backoff_conf, op).await?;

    // Stream to temp file while hashing
    let dir = local_path.parent().unwrap_or_else(|| Path::new("."));
    let mut temp_file = NamedTempFile::new_in(dir).map_err(InfraError::Io)?;
    let mut hasher = Sha256::new();
    let mut stream = res.bytes_stream();
    let mut total_bytes = 0u64;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(InfraError::Network)?;
        total_bytes += chunk.len() as u64;

        if total_bytes > MAX_INPUT_FILE_SIZE {
            return Err(InfraError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Download exceeded size limit of {} bytes",
                    MAX_INPUT_FILE_SIZE
                ),
            )));
        }

        temp_file.write_all(&chunk).map_err(InfraError::Io)?;
        hasher.update(&chunk);
    }

    // Verify Hash of Downloaded Content
    if let Some(expected) = expected_hash {
        let calculated = hex::encode(hasher.finalize());
        if calculated != expected {
            return Err(InfraError::HashMismatch {
                expected: expected.to_string(),
                actual: calculated,
            });
        }
    }

    // Atomic Rename
    temp_file
        .persist(local_path)
        .map_err(|e| InfraError::Io(e.error))?;

    Ok(())
}

/// Ensures that all files in a corpus bundle are downloaded and present locally.
pub async fn ensure_corpus_bundle(client: &HiveClient, corpus_name: &str) -> InfraResult<String> {
    let bundle_dir = if corpus_name == "default" {
        format!("data/corpora/{}", DEFAULT_CORPUS_ID)
    } else {
        format!("data/corpora/{}", corpus_name)
    };

    let files = [
        ASSET_1GRAMS_FILENAME,
        ASSET_2GRAMS_FILENAME,
        ASSET_3GRAMS_FILENAME,
        ASSET_WORDS_FILENAME,
    ];

    for f in files {
        let local_str = format!("{}/{}", bundle_dir, f);
        let local_path = Path::new(&local_str);
        let remote = client.url(&local_str);

        ensure_file(client, &remote, local_path, None).await?;
    }

    Ok(bundle_dir)
}

/// Ensures the specified cost matrix file is downloaded and present in the workspace.
pub async fn ensure_cost_matrix(
    client: &HiveClient,
    workspace_root: &Path,
    filename: &str,
) -> InfraResult<PathBuf> {
    let local_path = workspace_root.join(filename);
    let remote_path = format!("data/{}", filename);
    let url = client.url(&remote_path);

    ensure_file(client, &url, &local_path, None).await?;

    Ok(local_path)
}
