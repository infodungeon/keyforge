#![allow(clippy::print_stdout, clippy::print_stderr)]
// apps/keyforge-cli/src/cmd/auth.rs

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

use crate::constants::{CLI_CONFIG_FILENAME, CONFIG_DIR_NAME};
use crate::error::{CliError, CliResult as Result};
use clap::{Args, Subcommand};
use keyforge_boundary::SafePath;
use keyforge_infra::fs::io::atomic_write;
use serde::{Deserialize, Serialize};

#[derive(Args, Debug, Clone)]
pub struct AuthArgs {
    #[command(subcommand)]
    pub command: AuthCommands,

    #[arg(long, default_value = keyforge_model::constants::DEFAULT_HIVE_URL)]
    pub hive: String,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AuthCommands {
    /// Register a new user account
    Register {
        #[arg(short, long)]
        username: String,
    },
    /// Manually save an existing API Key
    Login {
        #[arg(short, long)]
        key: String,
    },
    /// Show current authentication status
    Whoami,
}

#[derive(Serialize, Deserialize)]
struct CliConfig {
    api_key: Option<String>,
}

fn get_config_path() -> Result<SafePath> {
    let base =
        dirs::config_dir().ok_or_else(|| CliError::Other("Could not find config dir".into()))?;
    let rel_dir =
        SafePath::try_from_str(CONFIG_DIR_NAME).map_err(|e| CliError::Other(e.to_string()))?;
    let dir = SafePath::from_trusted_root(&base, &rel_dir);

    std::fs::create_dir_all(dir.as_path()).map_err(CliError::Io)?;

    let path = dir
        .join(CLI_CONFIG_FILENAME)
        .map_err(|e| CliError::Other(e.to_string()))?;
    Ok(path)
}

fn save_key(key: &str) -> Result<()> {
    let path = get_config_path()?;
    let config = CliConfig {
        api_key: Some(key.to_string()),
    };
    let json = serde_json::to_string_pretty(&config)?;

    // Write file
    atomic_write(&path, json).map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;

    // Harden permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(path.as_path()) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o600); // User Read/Write ONLY
            if let Err(e) = std::fs::set_permissions(path.as_path(), perms) {
                eprintln!("⚠️  Warning: Failed to set secure permissions on config file: {e}");
            }
        }
    }

    Ok(())
}

pub fn load_key() -> Option<String> {
    // 1. Check Env Var (Highest Priority for Headless/CI)
    if let Ok(env_key) = std::env::var("KEYFORGE_API_KEY") {
        return Some(env_key);
    }

    // 2. Check Config File
    let path = get_config_path().ok()?;
    if !path.as_path().exists() {
        return None;
    }
    let content = keyforge_infra::fs::io::read_to_string_limited(
        &path,
        keyforge_model::constants::MAX_INPUT_FILE_SIZE,
    )
    .ok()?;
    let config: CliConfig = serde_json::from_str(&content).ok()?;
    config.api_key
}

pub async fn run(args: AuthArgs) -> Result<()> {
    match args.command {
        AuthCommands::Register { username } => {
            let client = reqwest::Client::new();
            let url = format!("{}/auth/register", args.hive);
            let hive = &args.hive;

            eprintln!("🌐 Registering '{username}' at {hive}...");

            let res = client
                .post(&url)
                .json(&serde_json::json!({ "username": username }))
                .send()
                .await
                .map_err(CliError::Network)?;

            if res.status().is_success() {
                let body: serde_json::Value = res.json().await.map_err(CliError::Network)?;
                if let Some(key) = body.get("api_key").and_then(|s| s.as_str()) {
                    save_key(key)?;
                    println!("✅ Registration successful!");
                    println!("🔑 API Key: {key}");
                    println!("   (Saved to config. You can now run searches as '{username}')");
                } else {
                    return Err(CliError::Other("Invalid server response".into()));
                }
            } else if res.status() == 409 {
                return Err(CliError::Other(format!(
                    "Username '{username}' is already taken."
                )));
            } else {
                let status = res.status();
                return Err(CliError::Other(format!("Registration failed: {status}")));
            }
        }
        AuthCommands::Login { key } => {
            save_key(&key)?;
            println!("✅ API Key saved securely.");
        }
        AuthCommands::Whoami => {
            if let Some(key) = load_key() {
                let masked = if key.len() > 8 {
                    format!("{}...{}", &key[0..6], &key[key.len() - 4..])
                } else {
                    "********".to_string()
                };
                println!("👤 Authenticated");
                println!("🔑 Key: {masked}");
            } else {
                println!("👤 Not logged in.");
            }
        }
    }
    Ok(())
}
