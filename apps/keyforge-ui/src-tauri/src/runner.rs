use crate::error::CommandError;
use keyforge_protocol::JobConfig;
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;
use tracing::warn;

#[derive(Debug)]
pub struct AgentRunner {
    app: AppHandle,
}

impl AgentRunner {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
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
                &temp_path.to_string_lossy(),
                layout,
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
                    if let Some(code) = status.code {
                        if code != 0 {
                             return Err(CommandError::Internal(format!("Agent exited with status: {}", code)));
                        }
                    } else {
                         return Err(CommandError::Internal("Agent terminated by signal".to_string()));
                    }
                }
                _ => {}
            }
        }
        
        Ok(output)
    }
}
