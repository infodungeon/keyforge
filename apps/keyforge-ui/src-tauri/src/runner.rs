use crate::error::CommandError;
use keyforge_protocol::JobConfig;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;
use tracing::warn;

#[derive(Debug)]
pub struct AgentRunner {
    app: AppHandle,
    data_dir: PathBuf,
}

impl AgentRunner {
    pub fn new(app: AppHandle, data_dir: PathBuf) -> Self {
        Self { app, data_dir }
    }

    pub async fn run_validation(&self, config: &JobConfig, layout: &str) -> Result<String, CommandError> {
        // Create temp file for JobConfig
        let temp_file = tempfile::NamedTempFile::new().map_err(|e| CommandError::Internal(e.to_string()))?;
        let temp_path = temp_file.path().to_path_buf();
        let json = serde_json::to_string(config).map_err(|e| CommandError::Internal(e.to_string()))?;
        tokio::fs::write(&temp_path, json).await.map_err(|e| CommandError::Internal(e.to_string()))?;

        // Spawn sidecar
        // args: ["score", temp_path, layout, "--data-dir", data_dir]
        let sidecar_command = self.app.shell().sidecar("keyforge-agent")
            .map_err(|e| CommandError::Internal(e.to_string()))?
            .args([
                "score",
                "--job-file",
                &temp_path.to_string_lossy(),
                "--layout",
                layout,
                "--data-dir",
                &self.data_dir.to_string_lossy()
            ]);

        let (mut rx, _child) = sidecar_command
            .spawn()
            .map_err(|e| CommandError::Internal(format!("Failed to spawn agent: {}", e)))?;

        let mut output = String::new();
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line) => {
                    let s = String::from_utf8_lossy(&line);
                    output.push_str(&s);
                }
                CommandEvent::Stderr(line) => {
                    let s = String::from_utf8_lossy(&line);
                    if !s.trim().is_empty() {
                         warn!("Agent stderr: {}", s.trim());
                    }
                }
                CommandEvent::Error(e) => {
                    return Err(CommandError::Internal(format!("Agent error: {}", e)));
                }
                CommandEvent::Terminated(status) => {
                    if !status.code.unwrap_or(0) == 0 {
                         return Err(CommandError::Internal(format!("Agent exited with status: {:?}", status)));
                    }
                }
                _ => {}
            }
        }
        
        Ok(output)
    }
}
