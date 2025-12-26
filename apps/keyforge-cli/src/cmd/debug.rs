use clap::{Args, Subcommand};
// use keyforge_export::viz::physics::generate_physics_svg;
use keyforge_infra::fs::io::read_to_string_limited;
use keyforge_protocol::constants::MAX_INPUT_FILE_SIZE;
use keyforge_protocol::geometry::KeyboardDefinition;
// use std::fs;
use std::path::{Path, PathBuf};

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

        #[arg(short, long, default_value = "debug_physics.svg")]
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

            let _def = KeyboardDefinition::parse(&content, None)
                .map_err(|e| format!("Failed to parse keyboard JSON: {}", e))?;

            /*
            let svg_content = generate_physics_svg(&def.geometry);
            fs::write(&output, svg_content).map_err(|e| format!("Failed to write SVG: {}", e))?;
            eprintln!("✅ Physics visualization saved to {:?}", output);
            */
            eprintln!("⚠️ Physics visualization is currently disabled.");
        }
    }
    Ok(())
}
