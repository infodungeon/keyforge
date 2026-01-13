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

use anyhow::Context;
use clap::{Parser, Subcommand};
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use std::path::PathBuf;
use tokio::sync::{broadcast, mpsc};
use tracing::{info, error};
use keyforge_agent::agent::errors::AgentError;
use keyforge_agent::models::{AgentConfig, PartialAgentConfig, SystemConfig};
use keyforge_protocol::JobConfig;

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(long, env = "KEYFORGE_HIVE_URL")]
    hive: Option<String>,

    #[arg(long, env = "KEYFORGE_CORES")]
    cores: Option<usize>,

    #[arg(long, env = "KEYFORGE_DATA_DIR")]
    data_dir: Option<PathBuf>,

    #[arg(long, env = "KEYFORGE_CONFIG")]
    config: Option<PathBuf>,

    #[arg(long, default_value_t = false)]
    skip_calibration: bool,
}

#[derive(Subcommand, Clone)]
enum Commands {
    /// Start as a long-running worker node connecting to a Hive.
    Worker,
    /// Run a single optimization job defined in a file.
    Run {
        /// Path to the JobConfig JSON file.
        job_file: PathBuf,
        /// Maximum time in seconds.
        #[arg(long)]
        timeout: Option<u64>,
    },
    /// Score a specific layout string against a JobConfig.
    Score {
        /// Path to the JobConfig JSON file.
        job_file: PathBuf,
        /// The layout string to score.
        layout: String,
        /// Maximum time in seconds.
        #[arg(long)]
        timeout: Option<u64>,
    },
    /// Run a physics benchmark using the environment from a JobConfig.
    Bench {
        /// Path to the JobConfig JSON file.
        job_file: PathBuf,
        /// Number of iterations.
        #[arg(long, default_value_t = keyforge_model::constants::DEFAULT_BENCHMARK_ITERATIONS)]
        iterations: usize,
    }
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
            if let Ok(cfg) = PartialAgentConfig::from_file(&path) {
                return Some(cfg);
            }
        }
    }
    None
}

async fn read_job_config(path: &PathBuf) -> anyhow::Result<JobConfig> {
    let content = if path.to_str() == Some("-") {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf).context("Failed to read from stdin")?;
        buf
    } else {
        tokio::fs::read_to_string(path).await
            .context(format!("Failed to read job file {:?}", path))?
    };
    serde_json::from_str(&content).context("Invalid Job JSON")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut config = AgentConfig::default();
    
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

    if let Some(h) = args.hive { config.hive_url = h; }
    if let Some(c) = args.cores { config.cores = c; }
    if let Some(d) = args.data_dir { config.data_dir = d; }
    if args.skip_calibration { config.calibration.duration_ms = 0; }

    let command = args.command.unwrap_or(Commands::Worker);
    let log_mode = match command {
        Commands::Worker => keyforge_agent::logging::LogMode::Standard,
        _ => keyforge_agent::logging::LogMode::JsonStderr,
    };
    
    keyforge_agent::logging::init_tracing(&config.logging.default_filter, log_mode);

    let hive_url = config.hive_url.clone();
    let data_dir = config.data_dir.clone();

    match command {
        Commands::Worker => {
            let signing_key = load_or_create_identity(&config.system)?;
            let public_key = VerifyingKey::from(&signing_key);
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(public_key.to_bytes());
            let pk_hash = hex::encode(hasher.finalize());
            let node_id = format!("{}{}", config.system.node_id_prefix, &pk_hash[0..8]);

            info!("agent starting in WORKER mode");
            info!(hive_url = %hive_url, "connecting to hive");
            info!(data_dir = ?data_dir, "data directory configured");

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
                        let _ = tx_clone.send(());
                    }
                    _ = sig_usr1.recv() => {
                        let _ = tx_clone.send(());
                    }
                }
            });

            keyforge_agent::run_worker(config, node_id, signing_key, rx).await;
        }
        Commands::Run { job_file, timeout } => {
            let job = read_job_config(&job_file).await?;

            if let Some(t) = timeout {
                config.compute.job_timeout_sec = t;
            }

            let (result_tx, mut result_rx) = mpsc::channel(1);
            
            let agent = keyforge_agent::agent::Agent::new(config.clone(), result_tx).await
                .map_err(|e| anyhow::anyhow!("Failed to init agent: {}", e))?;

            let (job_tx, job_rx) = mpsc::channel(1);
            let (_stop_tx, stop_rx) = mpsc::channel(1);
            
            let agent_handle = tokio::spawn(async move {
                agent.run(job_rx, stop_rx).await
            });

            let job_id = "local-job".to_string(); 
            job_tx.send((job_id, job)).await.ok();
            
            if let Some(result) = result_rx.recv().await {
                let json = serde_json::to_string(&result).unwrap();
                println!("{}", json);
            } else {
                error!("No result produced!");
                std::process::exit(1);
            }
            
            drop(job_tx);
            let _ = agent_handle.await;
        }
        Commands::Score { job_file, layout, timeout } => {
            // USE TRACING ERROR TO FORCE VISIBILITY
            error!("DEBUG_TRACE_ID_999: Scoring layout: '{}'", layout);

            let job = read_job_config(&job_file).await?;
            
            if let Some(t) = timeout {
                config.compute.job_timeout_sec = t;
            }

            let loader = keyforge_infra::FsProvider::new(data_dir.clone());
            let options = keyforge_runner::RunnerOptions {
                keycodes_file: config.compute.keycodes_file.clone(),
                ..Default::default()
            };

            let session = keyforge_runner::OptimizationRunner::prepare_session(
                &loader, &job, &options
            ).await?;

            // Try to resolve as layout name from definition first, then parse as raw string
            let layout_parsed = if let Some(layout_str) = job.definition.layouts.get(&layout) {
                keyforge_adapter::conversion::parse_layout_string(
                    layout_str, 
                    session.engine.key_count(), 
                    &session.registry
                ).map_err(|e| anyhow::anyhow!("Invalid layout in definition: {}", e))?
            } else {
                keyforge_adapter::conversion::parse_layout_string(
                    &layout, 
                    session.engine.key_count(), 
                    &session.registry
                ).map_err(|e| anyhow::anyhow!("Invalid layout string: {}", e))?
            };
            
            let report = session.engine.analyze(&layout_parsed);
            match report {
                Ok(r) => println!("{}", serde_json::to_string_pretty(&r)?),
                Err(e) => return Err(anyhow::anyhow!("Analysis failed: {:?}", e)),
            }
        }
        Commands::Bench { job_file, iterations } => {
            let job = read_job_config(&job_file).await?;
            
            let loader = keyforge_infra::FsProvider::new(data_dir.clone());
            let options = keyforge_runner::RunnerOptions {
                keycodes_file: config.compute.keycodes_file.clone(),
                ..Default::default()
            };

            let session = keyforge_runner::OptimizationRunner::prepare_session(
                &loader, &job, &options
            ).await?;

            let start = std::time::Instant::now();
            let mut score_sum = 0.0;
            
            let engine = session.engine;
            let default_layout = keyforge_model::Layout::new_unchecked(vec![keyforge_model::KeyCode(0); engine.key_count()]);

            for _ in 0..iterations {
                score_sum += engine.score(&default_layout)?;
            }
            
            let duration = start.elapsed();
            let kops = (iterations as f64 / duration.as_secs_f64()) / 1000.0;
            
            println!("{}", serde_json::json!({
                "iterations": iterations,
                "duration_ms": duration.as_millis(),
                "kops": kops,
                "checksum": score_sum
            }));
        }
    }

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

    let passphrase = if let Some(override_id) = &config.machine_id_override {
        override_id.clone()
    } else if let Ok(env_id) = std::env::var("KEYFORGE_MACHINE_ID") {
        env_id
    } else {
        machine_id_timeout_safe()
            .map_err(|e| AgentError::Identity(format!("Fatal: Could not derive machine ID. Secure fallback unavailable: {}", e)))?
    };

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
