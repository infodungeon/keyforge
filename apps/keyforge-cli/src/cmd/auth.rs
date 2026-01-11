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


use crate::error::{CliError, Result};
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::constants::{CONFIG_DIR_NAME, CLI_CONFIG_FILENAME};

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

fn get_config_path() -> Result<PathBuf> {
    let mut path =
        dirs::config_dir().ok_or_else(|| CliError::Other("Could not find config dir".into()))?;
    path.push(CONFIG_DIR_NAME);
    std::fs::create_dir_all(&path).map_err(CliError::Io)?;
    path.push(CLI_CONFIG_FILENAME);
    Ok(path)
}

fn save_key(key: &str) -> Result<()> {
    let path = get_config_path()?;
    let config = CliConfig {
        api_key: Some(key.to_string()),
    };
    let json = serde_json::to_string_pretty(&config)?;
    
    // Write file
    std::fs::write(&path, json).map_err(CliError::Io)?;

    // Harden permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(&path) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o600); // User Read/Write ONLY
            if let Err(e) = std::fs::set_permissions(&path, perms) {
                eprintln!("⚠️  Warning: Failed to set secure permissions on config file: {}", e);
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
    if !path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(path).ok()?;
    let config: CliConfig = serde_json::from_str(&content).ok()?;
    config.api_key
}

pub async fn run(args: AuthArgs) -> Result<()> {
    match args.command {
        AuthCommands::Register { username } => {
            let client = reqwest::Client::new();
            let url = format!("{}/auth/register", args.hive);

            eprintln!("🌐 Registering '{}' at {}...", username, args.hive);

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
                    println!("🔑 API Key: {}", key);
                    println!(
                        "   (Saved to config. You can now run searches as '{}')",
                        username
                    );
                } else {
                    return Err(CliError::Other("Invalid server response".into()));
                }
            } else if res.status() == 409 {
                return Err(CliError::Other(format!(
                    "Username '{}' is already taken.",
                    username
                )));
            } else {
                return Err(CliError::Other(format!(
                    "Registration failed: {}",
                    res.status()
                )));
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
                println!("🔑 Key: {}", masked);
            } else {
                println!("👤 Not logged in.");
            }
        }
    }
    Ok(())
}
