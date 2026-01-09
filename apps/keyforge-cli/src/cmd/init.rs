// apps/keyforge-cli/src/cmd/init.rs

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


use crate::error::CliError;
use clap::Args;
use keyforge_infra::init::{ensure_dir, USER_WORKSPACE_DIRS};
use std::path::PathBuf;

#[derive(Args, Debug, Clone)]
pub struct InitArgs {
    #[arg(default_value = ".")]
    pub path: PathBuf,

    #[arg(long, default_value = "http://localhost:3000")]
    pub hive: String,
}

pub async fn run(args: InitArgs) -> Result<(), CliError> {
    let root = args.path.join("data");
    eprintln!("🚀 Initializing KeyForge Workspace at {:?}", root);

    // 1. Create Structure (Client Policy: Workspace Only)
    for d in USER_WORKSPACE_DIRS {
        ensure_dir(&root, d)
            .map_err(|e| CliError::Workspace(format!("Failed to create {}: {}", d, e)))?;
    }

    // 2. Download Essentials (Online)
    eprintln!("🌐 Connecting to Hive at {}...", args.hive);
    let config = keyforge_infra::net::client::ClientConfig {
        base_url: args.hive.clone(),
        secret: None,
        ..Default::default()
    };
    let client = match keyforge_infra::HiveClient::new(config) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("⚠️  Could not connect to Hive: {}. Skipping downloads.", e);
            eprintln!("✅ Workspace initialized (offline mode).");
            return Ok(());
        }
    };

    // Use dynamic bootstrap instead of hardcoded list
    match keyforge_infra::net::sync::bootstrap_essentials(&client, &root).await {
        Ok(files) => {
            eprintln!("✅ Downloaded {} essential assets.", files.len());
        }
        Err(e) => {
            eprintln!("⚠️  Bootstrap failed: {}", e);
        }
    }

    eprintln!("✅ Workspace initialized.");
    Ok(())
}
