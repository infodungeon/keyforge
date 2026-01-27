// apps/keyforge-cli/src/cmd/list.rs

use clap::Args;
use comfy_table::Table;
use keyforge_adapter::loader::AssetLoader;
use keyforge_model::geometry::KeyboardDefinition;

#[derive(Args, Debug, Clone)]
pub struct ListArgs {
    /// Filter by type (e.g., "keyboard", "corpus").
    #[arg(short, long)]
    pub kind: Option<String>,
}

pub async fn run<L: AssetLoader + ?Sized>(
    args: ListArgs,
    loader: &L,
) -> crate::error::CliResult<()> {
    let kind = args.kind.unwrap_or_else(|| "keyboard".to_string());

    match kind.as_str() {
        "keyboard" | "keyboards" => list_keyboards(loader).await?,
        "corpus" | "corpora" => list_corpora(loader)?,
        _ => println!("Unknown asset kind: {kind}"),
    }

    Ok(())
}

async fn list_keyboards<L: AssetLoader + ?Sized>(loader: &L) -> crate::error::CliResult<()> {
    let keyboards = keyforge_infra::fs::listing::list_keyboards(loader.root())?;
    let mut table = Table::new();
    table.set_header(vec!["ID", "Name", "Type", "Author"]);

    for path in keyboards {
        let name = path.file_stem().unwrap_or_default().to_string_lossy();
        if let Ok(def) = loader.load::<KeyboardDefinition>(&name).await {
            table.add_row(vec![
                name.to_string(),
                def.meta.name.clone(),
                def.meta.kb_type.clone(),
                def.meta.author.clone(),
            ]);
        }
    }

    println!("{table}");
    Ok(())
}

fn list_corpora<L: AssetLoader + ?Sized>(loader: &L) -> crate::error::CliResult<()> {
    let corpora = keyforge_infra::fs::listing::list_corpora(loader.root())?;
    let mut table = Table::new();
    table.set_header(vec!["Filename", "Type", "Size"]);

    for path in corpora {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        let meta = std::fs::metadata(&path)?;
        let kind = if name.ends_with(".json") {
            "JSON"
        } else {
            "Text"
        };
        table.add_row(vec![
            name.to_string(),
            kind.to_string(),
            format!("{} bytes", meta.len()),
        ]);
    }

    println!("{table}");
    Ok(())
}
