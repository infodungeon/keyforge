// apps/keyforge-cli/src/cmd/list.rs

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


use clap::{Args, Subcommand};
use comfy_table::presets::ASCII_FULL;
use comfy_table::Table;
use keyforge_infra::listing::{
    list_corpora as ws_list_corpora, list_keyboards as ws_list_keyboards,
};
use keyforge_infra::FsProvider;
use keyforge_core::loader::AssetLoader;
use keyforge_model::constants::MAX_INPUT_FILE_SIZE;
use std::fs;
use std::path::Path;
use crate::constants::DEFAULT_LIST_LIMIT;
use serde::Deserialize;

#[derive(Args, Debug, Clone)]
pub struct ListArgs {
    #[command(subcommand)]
    pub command: ListCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ListCommands {
    Keyboards {
        #[arg(long, default_value_t = DEFAULT_LIST_LIMIT)]
        limit: usize,
    },
    Corpora {
        #[arg(long, default_value_t = DEFAULT_LIST_LIMIT)]
        limit: usize,
    },
    Layouts {
        #[arg(help = "Name of the keyboard file")]
        keyboard: String,
    },
}

// [Fixed] Partial struct for fast metadata extraction
#[derive(Deserialize)]
struct KeyboardHeader {
    meta: keyforge_model::geometry::KeyboardMeta,
    // Ignore geometry for list speed
}

pub async fn run(args: ListArgs, loader: &FsProvider) -> Result<(), Box<dyn std::error::Error>> {
    match args.command {
        ListCommands::Keyboards { limit } => list_keyboards(loader, limit).await,
        ListCommands::Corpora { limit } => list_corpora(loader, limit).await,
        ListCommands::Layouts { keyboard } => list_layouts(loader, &keyboard).await,
    }
}

fn apply_style(table: &mut Table) {
    table.load_preset(ASCII_FULL);
}

async fn list_keyboards(loader: &FsProvider, limit: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut table = Table::new();
    apply_style(&mut table);
    table.set_header(vec!["File", "Name", "Type", "Author"]);

    let names = ws_list_keyboards(&loader.root).map_err(|e| format!("Failed to list keyboards: {}", e))?;
    let count = names.len();
    for name in names.into_iter().take(limit) {
        if let Ok(def) = loader.load_keyboard(&name).await {
            table.add_row(vec![
                name,
                def.meta.name.clone(),
                def.meta.kb_type.clone(),
                def.meta.author.clone(),
            ]);
        }
    }
    println!("{table}");
    if count > limit {
        println!("... and {} more. Use --limit to see more.", count - limit);
    }
    Ok(())
}

async fn list_corpora(loader: &FsProvider, limit: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut table = Table::new();
    apply_style(&mut table);
    table.set_header(vec!["Category", "ID", "Size (1-grams)"]);

    let ids = ws_list_corpora(&loader.root).map_err(|e| format!("Failed to list corpora: {}", e))?;
    let count = ids.len();
    for id in ids.into_iter().take(limit) {
        let parts: Vec<&str> = id.split('/').collect();
        let (cat, name) = if parts.len() == 2 {
            (parts[0], parts[1])
        } else {
            ("root", parts[0])
        };

        let system_path = loader.root.join("system/corpora").join(&id);
        let user_path = loader.root.join("user/corpora").join(&id);
        
        let path = if user_path.exists() { user_path } else { system_path };
        let is_system = path.starts_with(loader.root.join("system"));
        let ext = if is_system { "mpk.zst" } else { "json" };

        let size = fs::metadata(path.join(format!("1grams.{}", ext)))
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

async fn list_layouts(loader: &FsProvider, kb_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let def = loader.load_keyboard(kb_name).await
        .map_err(|e| format!("Failed to load keyboard '{}': {}", kb_name, e))?;

    println!("Layouts for {}:", def.meta.name);
    let mut table = Table::new();
    apply_style(&mut table);
    table.set_header(vec!["Layout Name", "Length"]);

    for (name, layout) in &def.layouts {
        table.add_row(vec![name.clone(), layout.len().to_string()]);
    }
    println!("{table}");
    Ok(())
}
