// apps/keyforge-hive/src/services/runner.rs

use crate::error::AppError;
use keyforge_protocol::JobConfig;
use std::path::PathBuf;
use tokio::process::Command;
use tracing::{info, warn};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AgentRunner {
    pub data_dir: PathBuf,
}

impl AgentRunner {
    #[allow(dead_code)]
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    #[allow(dead_code)]
    pub async fn run_validation(
        &self,
        config: &JobConfig,
        layout: &str,
    ) -> Result<String, AppError> {
        // Create temp file for JobConfig
        let temp_file = tempfile::NamedTempFile::new().map_err(|e| AppError::Any(e.into()))?;
        let temp_path = temp_file.path().to_path_buf();
        let json = serde_json::to_string(config).map_err(AppError::Serde)?;
        tokio::fs::write(&temp_path, json)
            .await
            .map_err(|e| AppError::Any(e.into()))?;

        info!("Agent Runner spawning sidecar for validation...");

        // Ensure keyforge-agent is in PATH or provide absolute path.
        // For now, assume it's in PATH or sibling directory in release/debug.
        let output = Command::new("keyforge-agent")
            .args([
                "score",
                "--job-file",
                &temp_path.to_string_lossy(),
                "--layout",
                layout,
                "--data-dir",
                &self.data_dir.to_string_lossy(),
            ])
            .output()
            .await
            .map_err(|e| AppError::Any(anyhow::anyhow!("Failed to spawn agent: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Agent Runner failed: {}", stderr);
            return Err(AppError::Validation(format!(
                "Agent validation failed: {stderr}"
            )));
        }

        let stdout = String::from_utf8(output.stdout).map_err(|e| AppError::Any(e.into()))?;
        Ok(stdout)
    }
}
