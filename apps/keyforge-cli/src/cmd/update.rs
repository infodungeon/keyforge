// apps/keyforge-cli/src/cmd/update.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


use crate::error::{CliError, Result};
use crate::update::{check_for_update, perform_update, UpdateConfig};
use clap::Args;

#[derive(Args, Debug, Clone)]
pub struct UpdateArgs {
    /// Only check for updates, don't install
    #[arg(long)]
    pub check_only: bool,

    /// Force update even if already on latest version
    #[arg(long)]
    pub force: bool,
}

pub async fn run(args: UpdateArgs) -> Result<()> {
    let config = UpdateConfig::default();

    if args.check_only {
        eprintln!("🔍 Checking for updates...");
        match check_for_update(&config).await? {
            Some(version) => {
                eprintln!("✨ Update available: v{}", version);
                eprintln!("Run 'keyforge update' to install");
            }
            None => {
                eprintln!("✅ Already on the latest version");
            }
        }
        return Ok(());
    }

    // Perform update
    eprintln!("📦 Checking for updates...");
    match check_for_update(&config).await? {
        Some(version) => {
            eprintln!("✨ Update available: v{}", version);
            eprintln!("⬇️  Downloading and installing...");

            let new_version = tokio::task::spawn_blocking(move || perform_update(&config))
                .await
                .map_err(|e| CliError::Update(format!("Update task failed: {}", e)))??;

            eprintln!("✅ Successfully updated to v{}", new_version);
            eprintln!("🔄 Please restart the CLI to use the new version");
        }
        None => {
            if args.force {
                eprintln!("🔄 Force updating...");
                let new_version = tokio::task::spawn_blocking(move || perform_update(&config))
                    .await
                    .map_err(|e| CliError::Update(format!("Update task failed: {}", e)))??;
                eprintln!("✅ Reinstalled v{}", new_version);
            } else {
                eprintln!("✅ Already on the latest version");
            }
        }
    }

    Ok(())
}
