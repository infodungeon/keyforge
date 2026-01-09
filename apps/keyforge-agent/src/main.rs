// apps/keyforge-agent/src/main.rs

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


//! # KeyForge Agent Binary

use clap::Parser;
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use std::path::PathBuf;
use tokio::sync::broadcast;
use tracing::info;
use keyforge_agent::agent::errors::AgentError;
use keyforge_agent::models::{AgentConfig, PartialAgentConfig, SystemConfig};

#[derive(Parser)]
struct Args {
    #[arg(long, env = "KEYFORGE_HIVE_URL")]
    hive: Option<String>,

    #[arg(long, env = "KEYFORGE_CORES")]
    cores: Option<usize>,

    #[arg(long, env = "KEYFORGE_DATA_DIR")]
    data_dir: Option<PathBuf>,

    #[arg(long, env = "KEYFORGE_CONFIG")]
    config: Option<PathBuf>,
}

fn load_config_from_standard_paths(data_dir_override: Option<&PathBuf>) -> Option<PartialAgentConfig> {
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
            println!("Loading configuration from {:?}", path);
            match PartialAgentConfig::from_file(&path) {
                Ok(cfg) => return Some(cfg),
                Err(e) => eprintln!("Failed to parse config file {:?}: {}", path, e),
            }
        }
    }
    println!("No configuration file found in standard locations. Using defaults.");
    None
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut config = AgentConfig::default();
    
    // 1. Load config
    if let Some(config_path) = &args.config {
        match PartialAgentConfig::from_file(config_path) {
            Ok(file_cfg) => config.merge(file_cfg),
            Err(e) => {
                eprintln!("Failed to load config file {:?}: {}", config_path, e);
                std::process::exit(1);
            }
        }
    } else if let Some(file_cfg) = load_config_from_standard_paths(args.data_dir.as_ref()) {
        config.merge(file_cfg);
    }

    // 2. Override with CLI/Env
    if let Some(h) = args.hive { config.hive_url = h; }
    if let Some(c) = args.cores { config.cores = c; }
    if let Some(d) = args.data_dir { config.data_dir = d; }

    // 3. Init Logging with configured filter
    keyforge_agent::logging::init_tracing(&config.logging.default_filter);

    let hive_url = config.hive_url.clone();
    let data_dir = config.data_dir.clone();

    info!("agent starting");
    info!(hive_url = %hive_url, "connecting to hive");
    info!(data_dir = ?data_dir, "data directory configured");

    // 4. Identity Management
    let signing_key = load_or_create_identity(&config.system)?;

    let public_key = VerifyingKey::from(&signing_key);
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(public_key.to_bytes());
    let pk_hash = hex::encode(hasher.finalize());
    let node_id = format!("{}{}", config.system.node_id_prefix, &pk_hash[0..8]);

    // 5. Signal Handling
    let (tx, rx) = broadcast::channel(config.system.shutdown_channel_capacity);
    #[cfg(unix)]
    let mut sig_usr1 =
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::user_defined1())
            .map_err(|e| anyhow::anyhow!("failed to register SIGUSR1: {}", e))?;
    #[cfg(not(unix))]
    let mut sig_usr1 = std::future::pending::<()>();

    let tx_clone = tx.clone();
    tokio::spawn(async move {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("received ctrl-c, initiating shutdown");
                let _ = tx_clone.send(());
            }
            _ = sig_usr1.recv() => {
                info!("received SIGUSR1, initiating graceful drain");
                let _ = tx_clone.send(());
            }
        }
    });

    // 6. Run Worker
    keyforge_agent::run_worker(config, node_id, signing_key, rx).await;

    info!("agent exited cleanly");
    Ok(())
}

fn load_or_create_identity(config: &SystemConfig) -> Result<SigningKey, AgentError> {
    let mut path = dirs::config_dir().ok_or_else(|| AgentError::Identity("could not find config directory".into()))?;
    path.push(&config.config_dir_name);

    if !path.exists() {
        std::fs::create_dir_all(&path)
            .map_err(|e| AgentError::Identity(format!("failed to create config dir: {}", e)))?;
    }

    path.push(&config.key_file_name);

    let passphrase = machine_id_timeout_safe()
        .map_err(|e| AgentError::Identity(format!("Fatal: Could not derive machine ID. Secure fallback unavailable: {}", e)))?;

    if path.exists() {
        let file =
            std::fs::File::open(&path).map_err(|e| AgentError::Identity(format!("failed to open key file: {}", e)))?;
        let decryptor =
            age::Decryptor::new(file).map_err(|e| AgentError::Identity(format!("age decryptor error: {}", e)))?;

        let identity = age::scrypt::Identity::new(passphrase.clone().into());
        let mut reader = decryptor
            .decrypt(std::iter::once(&identity as &dyn age::Identity))
            .map_err(|e| AgentError::Identity(format!("decryption failed: {}", e)))?;

        let mut decrypted = Vec::new();
        use std::io::Read;
        reader
            .read_to_end(&mut decrypted)
            .map_err(|e| AgentError::Identity(format!("failed to read decrypted key: {}", e)))?;

        let array: [u8; 32] = decrypted
            .try_into()
            .map_err(|_| AgentError::Identity("invalid key file length (expected 32 bytes)".into()))?;

        info!(path = ?path, "loaded encrypted identity");
        Ok(SigningKey::from_bytes(&array))
    } else {
        let mut csprng = OsRng;
        let key = SigningKey::generate(&mut csprng);

        let encryptor = age::Encryptor::with_user_passphrase(passphrase.into());

        let mut output = Vec::new();
        let mut writer = encryptor
            .wrap_output(&mut output)
            .map_err(|e| AgentError::Identity(format!("failed to initialize age writer: {}", e)))?;
        use std::io::Write;
        writer
            .write_all(&key.to_bytes())
            .map_err(|e| AgentError::Identity(format!("failed to write to age writer: {}", e)))?;
        writer
            .finish()
            .map_err(|e| AgentError::Identity(format!("failed to finish age encryption: {}", e)))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&path)
                .map_err(|e| AgentError::Identity(format!("failed to create hardened key file: {}", e)))?;
            file.write_all(&output)
                .map_err(|e| AgentError::Identity(format!("failed to save encrypted key: {}", e)))?;
        }
        #[cfg(not(unix))]
        {
            std::fs::write(&path, &output)
                .map_err(|e| AgentError::Identity(format!("failed to save encrypted key: {}", e)))?;
        }

        info!(path = ?path, "generated new encrypted identity");
        Ok(key)
    }
}

fn machine_id_timeout_safe() -> Result<String, String> {
    machine_uid::get().map_err(|e| {
        format!(
            "Security Requirement: Could not derive unique machine ID: {}",
            e
        )
    })
}
