// apps/keyforge-cli/src/runner.rs

use crate::error::CliError;
use keyforge_protocol::JobConfig;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tracing::{debug, info};

/// Locates the `keyforge-agent` binary.
/// 1. Checks `KEYFORGE_AGENT_PATH` env var.
/// 2. Checks sibling directory of current executable.
/// 3. Checks PATH.
fn find_agent() -> Result<PathBuf, CliError> {
    if let Ok(path) = std::env::var("KEYFORGE_AGENT_PATH") {
        return Ok(PathBuf::from(path));
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let sibling = dir.join("keyforge-agent");
            if sibling.exists() {
                return Ok(sibling);
            }
            // Windows check
            let sibling_exe = dir.join("keyforge-agent.exe");
            if sibling_exe.exists() {
                return Ok(sibling_exe);
            }
        }
    }

    which::which("keyforge-agent")
        .map_err(|_| CliError::Other("Could not find 'keyforge-agent' binary. Please ensure it is installed or set KEYFORGE_AGENT_PATH.".into()))
}

pub struct AgentRunner {
    agent_path: PathBuf,
    data_dir: PathBuf,
}

impl AgentRunner {
    pub fn new(data_dir: PathBuf) -> Result<Self, CliError> {
        let agent_path = find_agent()?;
        debug!("Using agent at: {:?}", agent_path);
        Ok(Self {
            agent_path,
            data_dir,
        })
    }

    /// Runs the agent in 'run' mode for a full optimization job.
    pub fn run_search(&self, config: &JobConfig) -> Result<(), CliError> {
        let temp_file = tempfile::NamedTempFile::new().map_err(CliError::Io)?;
        let temp_path = temp_file.path().to_path_buf();
        
        let json = serde_json::to_string(config).map_err(CliError::Json)?;
        std::fs::write(&temp_path, json).map_err(CliError::Io)?;

        let mut cmd = Command::new(&self.agent_path);
        cmd.arg("--data-dir").arg(&self.data_dir);
        cmd.arg("run");
        cmd.arg(temp_path);
        
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        info!("Spawning agent...");
        let mut child = cmd.spawn().map_err(CliError::Io)?;

        // Stream Output
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                match line {
                    Ok(l) => println!("{}", l),
                    Err(_) => break,
                }
            }
        }

        let status = child.wait().map_err(CliError::Io)?;
        if !status.success() {
            return Err(CliError::Other(format!("Agent exited with status: {}", status)));
        }

        Ok(())
    }

    /// Runs the agent in 'bench' mode.
    pub fn run_benchmark(&self, config: &JobConfig, iterations: usize) -> Result<(), CliError> {
        let temp_file = tempfile::NamedTempFile::new().map_err(CliError::Io)?;
        let temp_path = temp_file.path().to_path_buf();
        
        let json = serde_json::to_string(config).map_err(CliError::Json)?;
        std::fs::write(&temp_path, json).map_err(CliError::Io)?;

        let mut cmd = Command::new(&self.agent_path);
        cmd.arg("--data-dir").arg(&self.data_dir);
        cmd.arg("bench");
        cmd.arg(temp_path);
        cmd.arg("--iterations").arg(iterations.to_string());

        let status = cmd.status().map_err(CliError::Io)?;
        if !status.success() {
            return Err(CliError::Other("Benchmark failed".into()));
        }
        Ok(())
    }

    /// Runs the agent in 'score' mode.
    pub fn run_validation(&self, config: &JobConfig, layout: &str) -> Result<(), CliError> {
        let temp_file = tempfile::NamedTempFile::new().map_err(CliError::Io)?;
        let temp_path = temp_file.path().to_path_buf();
        
        let json = serde_json::to_string(config).map_err(CliError::Json)?;
        std::fs::write(&temp_path, json).map_err(CliError::Io)?;

        let mut cmd = Command::new(&self.agent_path);
        cmd.arg("--data-dir").arg(&self.data_dir);
        cmd.arg("score");
        cmd.arg(temp_path);
        cmd.arg(layout);

        let status = cmd.status().map_err(CliError::Io)?;
        if !status.success() {
            return Err(CliError::Other("Validation failed".into()));
        }
        Ok(())
    }
}
