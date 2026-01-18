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
    // 1. Check existing file integrity with Metadata Optimization
    if local_path.exists() {
        if let Some(hash) = expected_hash {
            // Check for sidecar hash file to avoid re-reading large files
            let sidecar_path = local_path.with_extension(format!(
                "{}.sha256",
                local_path.extension().and_then(|s| s.to_str()).unwrap_or("")
            ));

            let mut trusted = false;
            if sidecar_path.exists() {
                if let (Ok(meta_data), Ok(meta_side)) = (
                    tokio::fs::metadata(local_path).await,
                    tokio::fs::metadata(&sidecar_path).await,
                ) {
                    if let (Ok(mtime_data), Ok(mtime_side)) = (meta_data.modified(), meta_side.modified()) {
                        // If sidecar is newer than data, and contains the expected hash, we trust it.
                        if mtime_side > mtime_data {
                            if let Ok(content) = tokio::fs::read_to_string(&sidecar_path).await {
                                if content.trim() == hash {
                                    trusted = true;
                                }
                            }
                        }
                    }
                }
            }

            if trusted {
                return Ok(());
            }

            // Fallback: Full content verification
            let file = tokio::fs::File::open(local_path).await.map_err(InfraError::Io)?;
            let mut reader = tokio::io::BufReader::new(file);
            let mut hasher = Sha256::new();
            let mut buffer = [0u8; 8192];

            use tokio::io::AsyncReadExt;
            loop {
                let n = reader.read(&mut buffer).await.map_err(InfraError::Io)?;
                if n == 0 { break; }
                hasher.update(&buffer[..n]);
            }

            let calculated = hex::encode(hasher.finalize());

            if calculated == hash {
                // Update sidecar for next time
                if let Err(e) = tokio::fs::write(&sidecar_path, hash).await {
                    warn!("Failed to write sidecar hash: {}", e);
                }
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
            }
            return Err(backoff::Error::permanent(InfraError::Network(err)));
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
                                    "Remote file exceeds size limit ({len} > {MAX_INPUT_FILE_SIZE})"
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
                    "Download exceeded size limit of {MAX_INPUT_FILE_SIZE} bytes"
                ),
            )));
        }

        temp_file.write_all(&chunk).map_err(InfraError::Io)?;
        hasher.update(&chunk);
    }

    // Verify Hash of Downloaded Content
    let calculated = hex::encode(hasher.finalize());
    if let Some(expected) = expected_hash {
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

    // Write sidecar
    if expected_hash.is_some() {
        let sidecar_path = local_path.with_extension(format!(
            "{}.sha256",
            local_path.extension().and_then(|s| s.to_str()).unwrap_or("")
        ));
        if let Err(e) = tokio::fs::write(&sidecar_path, calculated).await {
             warn!("Failed to write sidecar hash: {}", e);
        }
    }

    Ok(())
}

/// Ensures that all files in a corpus bundle are downloaded and present locally.
pub async fn ensure_corpus_bundle(client: &HiveClient, corpus_name: &str) -> InfraResult<String> {
    let bundle_dir = if corpus_name == "default" {
        format!("data/corpora/{DEFAULT_CORPUS_ID}")
    } else {
        format!("data/corpora/{corpus_name}")
    };

    let files = [
        ASSET_1GRAMS_FILENAME,
        ASSET_2GRAMS_FILENAME,
        ASSET_3GRAMS_FILENAME,
        ASSET_WORDS_FILENAME,
    ];

    for f in files {
        let local_str = format!("{bundle_dir}/{f}");
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
    let remote_path = format!("data/{filename}");
    let url = client.url(&remote_path);

    ensure_file(client, &url, &local_path, None).await?;

    Ok(local_path)
}
