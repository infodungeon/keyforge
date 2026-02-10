// apps/keyforge-cli/src/cmd/list.rs

use clap::Args;
use comfy_table::Table;
use keyforge_adapter::loader::AssetLoader;

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

    for id in keyboards {
        if let Ok(def_dto) = loader
            .load::<keyforge_protocol::KeyboardDefinitionDto>(&id)
            .await
        {
            table.add_row(vec![
                id,
                def_dto.meta.name.clone(),
                def_dto.meta.kb_type.clone(),
                def_dto.meta.author.clone(),
            ]);
        }
    }

    println!("{table}");
    Ok(())
}

fn list_corpora<L: AssetLoader + ?Sized>(loader: &L) -> crate::error::CliResult<()> {
    let corpora = keyforge_infra::fs::listing::list_corpora(loader.root())?;
    let mut table = Table::new();
    table.set_header(vec!["ID", "Type"]);

    for id in corpora {
        let kind = if std::path::Path::new(&id)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        {
            "JSON"
        } else {
            "Text"
        };
        table.add_row(vec![id, kind.to_string()]);
    }

    println!("{table}");
    Ok(())
}
