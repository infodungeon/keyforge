use crate::error::{CliError, Result};
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Args, Debug, Clone)]
pub struct AuthArgs {
    #[command(subcommand)]
    pub command: AuthCommands,

    #[arg(long, default_value = "http://localhost:3000")]
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
    path.push("keyforge");
    std::fs::create_dir_all(&path).map_err(CliError::Io)?;
    path.push("cli.json");
    Ok(path)
}

fn save_key(key: &str) -> Result<()> {
    let path = get_config_path()?;
    let config = CliConfig {
        api_key: Some(key.to_string()),
    };
    let json = serde_json::to_string_pretty(&config)?;
    std::fs::write(path, json).map_err(CliError::Io)?;
    Ok(())
}

pub fn load_key() -> Option<String> {
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
            println!("✅ API Key saved.");
        }
        AuthCommands::Whoami => {
            if let Some(key) = load_key() {
                let masked = format!("{}...{}", &key[0..6], &key[key.len() - 4..]);
                println!("👤 Authenticated");
                println!("🔑 Key: {}", masked);
            } else {
                println!("👤 Not logged in.");
            }
        }
    }
    Ok(())
}
