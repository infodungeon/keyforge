// apps/keyforge-cli/src/cmd/debug.rs

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


use clap::{Args, Subcommand};
use keyforge_export::viz::physics::generate_physics_svg;
use keyforge_infra::fs::io::read_to_string_limited;
use keyforge_model::constants::MAX_INPUT_FILE_SIZE;
use keyforge_model::geometry::KeyboardDefinition;
use std::fs;
use std::path::{Path, PathBuf};
use crate::constants::DEFAULT_DEBUG_OUTPUT;

#[derive(Args, Debug, Clone)]
pub struct DebugArgs {
    #[command(subcommand)]
    pub command: DebugCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum DebugCommands {
    Physics {
        #[arg(short, long)]
        keyboard: String,

        #[arg(short, long, default_value = DEFAULT_DEBUG_OUTPUT)]
        output: PathBuf,
    },
}

pub fn run(args: DebugArgs, root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    match args.command {
        DebugCommands::Physics { keyboard, output } => {
            eprintln!("🔬 Analyzing Physics Model for '{}'...", keyboard);

            if let Some(parent) = output.parent() {
                if !parent.as_os_str().is_empty() && !parent.exists() {
                    return Err(format!("Output directory does not exist: {:?}", parent).into());
                }
            }

            let path = crate::cli_parsers::resolve_path(&keyboard, Some("keyboards"), root)?;

            let content = read_to_string_limited(&path, MAX_INPUT_FILE_SIZE)
                .map_err(|e| format!("Failed to read keyboard file: {}", e))?;

            let def = KeyboardDefinition::parse(&content, None)
                .map_err(|e| format!("Failed to parse keyboard JSON: {}", e))?;

            
            let svg_content = generate_physics_svg(&def.geometry);
            fs::write(&output, svg_content).map_err(|e| format!("Failed to write SVG: {}", e))?;
            eprintln!("✅ Physics visualization saved to {:?}", output);
            
        }
    }
    Ok(())
}
