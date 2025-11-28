use clap::{Parser, Subcommand};
use keyforge::scorer::Scorer;
use std::sync::Arc;

mod cmd;
mod reports;

// Re-export for CLI usage
use cmd::search::SearchArgs;
use cmd::validate::ValidateArgs;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(global = true, short, long, default_value = "data/cost_matrix.csv")]
    cost: String,

    #[arg(global = true, short, long, default_value = "data/ngrams-all.tsv")]
    ngrams: String,

    #[arg(global = true, long)]
    geometry: Option<String>,

    #[arg(global = true, long, default_value_t = false)]
    debug: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Search(SearchArgs),
    Validate(ValidateArgs),
}

fn main() {
    let cli = Cli::parse();
    println!("\n🚀 Initializing KeyForge Core...");

    // Pre-load configuration to initialize Scorer
    let (config, command) = match cli.command {
        Commands::Search(args) => (args.config.clone(), Commands::Search(args)),
        Commands::Validate(args) => (args.config.clone(), Commands::Validate(args)),
    };

    if cli.debug {
        println!("🔧 Configuration Loaded:");
        println!("    Epochs: {}", config.search.search_epochs);
    }

    let scorer = Arc::new(Scorer::new(
        &cli.cost,
        &cli.ngrams,
        &cli.geometry,
        config,
        cli.debug,
    ));

    match command {
        Commands::Search(args) => cmd::search::run(args, scorer, cli.debug),
        Commands::Validate(args) => cmd::validate::run(args, scorer),
    }
}
