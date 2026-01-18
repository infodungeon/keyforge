use std::path::PathBuf;
use anyhow::Context;
use keyforge_protocol::JobConfig;
use crate::models::PartialAgentConfig;

#[must_use] 
pub fn load_config_from_standard_paths(data_dir_override: Option<&PathBuf>) -> Option<PartialAgentConfig> {
    let mut candidates = vec![
        PathBuf::from("agent.mpk.zst"),
        PathBuf::from("agent.toml"),
        PathBuf::from("agent.json"),
        dirs::config_dir().unwrap_or_default().join("keyforge/agent.mpk.zst"),
        dirs::config_dir().unwrap_or_default().join("keyforge/agent.toml"),
        dirs::config_dir().unwrap_or_default().join("keyforge/agent.json"),
        PathBuf::from("/etc/keyforge/agent.mpk.zst"),
        PathBuf::from("/etc/keyforge/agent.toml"),
        PathBuf::from("/etc/keyforge/agent.json"),
    ];

    if let Some(data_dir) = data_dir_override {
        candidates.insert(0, data_dir.join("user/config/agent.mpk.zst"));
        candidates.insert(1, data_dir.join("user/config/agent.toml"));
        candidates.insert(2, data_dir.join("user/config/agent.json"));
        candidates.insert(3, data_dir.join("system/config/agent.mpk.zst"));
        candidates.insert(4, data_dir.join("system/config/agent.toml"));
        candidates.insert(5, data_dir.join("system/config/agent.json"));
    } else if let Ok(env_dir) = std::env::var("KEYFORGE_DATA_DIR") {
        let path = PathBuf::from(env_dir);
        candidates.insert(0, path.join("user/config/agent.mpk.zst"));
        candidates.insert(1, path.join("user/config/agent.toml"));
        candidates.insert(2, path.join("user/config/agent.json"));
        candidates.insert(3, path.join("system/config/agent.mpk.zst"));
        candidates.insert(4, path.join("system/config/agent.toml"));
        candidates.insert(5, path.join("system/config/agent.json"));
    }

    for path in candidates {
        if path.exists() {
            if let Ok(cfg) = PartialAgentConfig::from_file(&path) {
                return Some(cfg);
            }
        }
    }
    None
}

pub async fn read_job_config(path: &PathBuf) -> anyhow::Result<JobConfig> {
    let content = if path.to_str() == Some("-") {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf).context("Failed to read from stdin")?;
        buf
    } else {
        tokio::fs::read_to_string(path).await
            .context(format!("Failed to read job file {path:?}"))?
    };
    serde_json::from_str(&content).context("Invalid Job JSON")
}
