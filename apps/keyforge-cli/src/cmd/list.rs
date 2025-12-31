use crate::cli_parsers::resolve_path;
use clap::{Args, Subcommand};
use comfy_table::presets::ASCII_FULL;
use comfy_table::Table;
use keyforge_infra::fs::io::read_to_string_limited;
use keyforge_infra::listing::{
    list_corpora as ws_list_corpora, list_keyboards as ws_list_keyboards,
};
use keyforge_model::constants::MAX_INPUT_FILE_SIZE;
use keyforge_model::geometry::KeyboardDefinition;
use std::fs;
use std::path::Path;

#[derive(Args, Debug, Clone)]
pub struct ListArgs {
    #[command(subcommand)]
    pub command: ListCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ListCommands {
    /// List available Keyboards
    Keyboards {
        #[arg(
            long,
            default_value_t = 50,
            help = "Limit the number of rows displayed"
        )]
        limit: usize,
    },
    /// List available Corpora
    Corpora {
        #[arg(
            long,
            default_value_t = 50,
            help = "Limit the number of rows displayed"
        )]
        limit: usize,
    },
    /// List Layouts defined within a specific Keyboard file
    Layouts {
        #[arg(help = "Name of the keyboard file (e.g. ortho_30)")]
        keyboard: String,
    },
}

pub fn run(args: ListArgs, root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    match args.command {
        ListCommands::Keyboards { limit } => list_keyboards(root, limit),
        ListCommands::Corpora { limit } => list_corpora(root, limit),
        ListCommands::Layouts { keyboard } => list_layouts(root, &keyboard),
    }
}

// Helper to check for NO_COLOR env var
fn apply_style(table: &mut Table) {
    table.load_preset(ASCII_FULL);
}

fn list_keyboards(root: &Path, limit: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut table = Table::new();
    apply_style(&mut table);
    table.set_header(vec!["File", "Name", "Type", "Keys"]);

    let names = ws_list_keyboards(root).map_err(|e| format!("Failed to list keyboards: {}", e))?;
    let count = names.len();
    for name in names.into_iter().take(limit) {
        let p = root.join("keyboards").join(format!("{}.json", name));
        if let Ok(content) = read_to_string_limited(&p, MAX_INPUT_FILE_SIZE) {
            if let Ok(def) = serde_json::from_str::<KeyboardDefinition>(&content) {
                table.add_row(vec![
                    name,
                    def.meta.name,
                    def.meta.kb_type,
                    def.geometry.keys.len().to_string(),
                ]);
            }
        }
    }
    println!("{table}");
    if count > limit {
        println!("... and {} more. Use --limit to see more.", count - limit);
    }
    Ok(())
}

fn list_corpora(root: &Path, limit: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut table = Table::new();
    apply_style(&mut table);
    table.set_header(vec!["Category", "ID", "Size (1-grams)"]);

    let ids = ws_list_corpora(root).map_err(|e| format!("Failed to list corpora: {}", e))?;
    let count = ids.len();
    for id in ids.into_iter().take(limit) {
        let parts: Vec<&str> = id.split('/').collect();
        let (cat, name) = if parts.len() == 2 {
            (parts[0], parts[1])
        } else {
            ("root", parts[0])
        };

        let path = if parts.len() == 2 {
            root.join("corpora").join(cat).join(name)
        } else {
            root.join("corpora").join(name)
        };

        let size = fs::metadata(path.join("1grams.json"))
            .map(|m| m.len())
            .unwrap_or(0);

        table.add_row(vec![
            cat.to_string(),
            name.to_string(),
            format!("{:.2} MB", size as f64 / 1024.0 / 1024.0),
        ]);
    }
    println!("{table}");
    if count > limit {
        println!("... and {} more. Use --limit to see more.", count - limit);
    }
    Ok(())
}

fn list_layouts(root: &Path, kb_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = resolve_path(kb_name, Some("keyboards"), root)?;

    let content = read_to_string_limited(&path, MAX_INPUT_FILE_SIZE)
        .map_err(|e| format!("Failed to read keyboard file: {}", e))?;

    let def = serde_json::from_str::<KeyboardDefinition>(&content)
        .map_err(|e| format!("Failed to parse keyboard definition: {}", e))?;

    println!("Layouts for {}:", def.meta.name);
    let mut table = Table::new();
    apply_style(&mut table);
    table.set_header(vec!["Layout Name", "Length"]);

    for (name, layout) in def.layouts {
        table.add_row(vec![name, layout.len().to_string()]);
    }
    println!("{table}");
    Ok(())
}
