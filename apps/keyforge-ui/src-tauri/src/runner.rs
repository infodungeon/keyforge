use crate::error::CommandError;
use crate::utils::get_data_dir;
use keyforge_protocol::JobConfig;
use tauri::AppHandle;
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;
use tracing::{debug, error, info, warn};

#[derive(Debug)]
pub struct AgentRunner {
    app: AppHandle,
}

impl AgentRunner {
    #[must_use]
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    /// Runs the validation sidecar for a given layout.
    ///
    /// # Errors
    ///
    /// Returns `CommandError` if the sidecar fails to spawn, execute, or returns a non-zero exit code.
    pub async fn run_validation(
        &self,
        config: &JobConfig,
        layout: &str,
    ) -> Result<String, CommandError> {
        // Create temp file for JobConfig
        let temp_file = tempfile::NamedTempFile::new()?;
        let temp_path = temp_file.path().to_path_buf();
        let json = serde_json::to_string(config)?;
        tokio::fs::write(&temp_path, json).await?;

        let data_dir = get_data_dir(&self.app)?;

        // Spawn sidecar
        // args: ["--data-dir", data_dir, "score", temp_path, layout]
        // Note: Global args must come before the subcommand
        let sidecar_command = self.app.shell().sidecar("keyforge-agent")?.args([
            "--data-dir",
            &data_dir.to_string_lossy(),
            "score",
            &temp_path.to_string_lossy(),
            layout,
        ]);

        let (mut rx, _child) = sidecar_command
            .spawn()
            .map_err(|e| CommandError::Internal(format!("Failed to spawn agent: {e}")))?;

        let mut output = String::new();
        let max_output_size = 1024 * 1024; // 1MB Limit

        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line) => {
                    let s = String::from_utf8_lossy(&line);
                    if output.len() + s.len() < max_output_size {
                        output.push_str(&s);
                    } else if output.len() < max_output_size {
                        let remaining = max_output_size - output.len();
                        output.push_str(&s[..remaining]);
                        output.push_str("\n... [Truncated due to size limit]");
                    }
                }
                CommandEvent::Stderr(line) => {
                    let s = String::from_utf8_lossy(&line);
                    let trimmed = s.trim();
                    if !trimmed.is_empty() {
                        // Try to parse structured log
                        if let Ok(log) = serde_json::from_str::<serde_json::Value>(trimmed) {
                            if let Some(level) = log.get("level").and_then(|v| v.as_str()) {
                                match level {
                                    "ERROR" => error!("Agent: {}", trimmed),
                                    "WARN" => warn!("Agent: {}", trimmed),
                                    "INFO" => info!("Agent: {}", trimmed),
                                    _ => debug!("Agent: {}", trimmed),
                                }
                                continue;
                            }
                        }
                        // Fallback for unstructured text
                        if trimmed.contains("error") || trimmed.contains("Error") {
                            warn!("Agent stderr: {}", trimmed);
                        } else {
                            debug!("Agent stderr: {}", trimmed);
                        }
                    }
                }
                CommandEvent::Error(e) => {
                    return Err(CommandError::Internal(format!("Agent error: {e}")));
                }
                CommandEvent::Terminated(status) => {
                    if let Some(code) = status.code {
                        if code != 0 {
                            return Err(CommandError::Internal(format!(
                                "Agent exited with status: {code}"
                            )));
                        }
                    } else {
                        return Err(CommandError::Internal(
                            "Agent terminated by signal".to_string(),
                        ));
                    }
                }
                _ => {}
            }
        }

        Ok(output)
    }
}
