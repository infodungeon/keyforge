use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use keyforge_core::geometry::KeyboardDefinition;
use keyforge_core::keycodes::KeycodeRegistry;
use keyforge_core::scorer::Scorer;
use std::path::Path;
use std::process;
use std::sync::Arc;
use tracing::{error, info, warn};

mod cmd;
mod reports;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(global = true, short, long, default_value = "data/cost_matrix.csv")]
    cost: String,

    #[arg(global = true, short, long, default_value = "data/ngrams-all.tsv")]
    ngrams: String,

    #[arg(
        global = true,
        short = 'k',
        long,
        default_value = "data/keyboards/ortho_30.json"
    )]
    keyboard: String,

    #[arg(global = true, long)]
    weights: Option<String>,

    #[arg(global = true, long, default_value_t = false)]
    debug: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Search(cmd::search::SearchArgs),
    Validate(cmd::validate::ValidateArgs),
}

fn main() {
    tracing_subscriber::fmt::init();

    let matches = Cli::command().get_matches();
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());

    info!("🚀 Initializing KeyForge Core...");

    info!("📂 Loading Keyboard: {}", cli.keyboard);
    let kb_def = KeyboardDefinition::load_from_file(&cli.keyboard).unwrap_or_else(|e| {
        error!("{}", e);
        process::exit(1);
    });

    let (mut config, cli_weights_ref, sub_matches) = match &cli.command {
        Commands::Search(args) => (
            args.config.clone(),
            &args.config.weights,
            matches.subcommand_matches("search").unwrap(),
        ),
        Commands::Validate(args) => (
            args.config.clone(),
            &args.config.weights,
            matches.subcommand_matches("validate").unwrap(),
        ),
    };

    let weights_path_str = if let Some(path) = &cli.weights {
        Some(path.clone())
    } else {
        let filename = match kb_def.meta.kb_type.as_str() {
            "ortho" | "column_staggered" => Some("ortho_split.json"),
            "row_staggered" => Some("row_stagger.json"),
            _ => None,
        };

        if let Some(name) = filename {
            let path = format!("data/weights/{}", name);
            if Path::new(&path).exists() {
                Some(path)
            } else {
                warn!(
                    "⚠️  Keyboard type '{}' maps to '{}', but file not found.",
                    kb_def.meta.kb_type, path
                );
                None
            }
        } else {
            None
        }
    };

    if let Some(path) = weights_path_str {
        info!("⚖️  Loading Weights from: {}", path);
        let mut file_weights = keyforge_core::config::ScoringWeights::load_from_file(&path);
        file_weights.merge_from_cli(cli_weights_ref, sub_matches);
        config.weights = file_weights;
    } else {
        warn!("⚠️  No external weights loaded. Using embedded defaults.");
    }

    let scorer_result = Scorer::new(&cli.cost, &cli.ngrams, &kb_def.geometry, config, cli.debug);

    let scorer = match scorer_result {
        Ok(s) => Arc::new(s),
        Err(e) => {
            error!("\n❌ FATAL ERROR INITIALIZING SCORER:");
            error!("   {}", e);
            process::exit(1);
        }
    };

    // Load Keycode Registry
    let registry_path = "data/keycodes.json";
    let registry = if Path::new(registry_path).exists() {
        info!("🔑 Loading Keycodes: {}", registry_path);
        KeycodeRegistry::load_from_file(registry_path).unwrap_or_else(|e| {
            warn!("Failed to load keycodes: {}. Using defaults.", e);
            KeycodeRegistry::new_with_defaults()
        })
    } else {
        warn!("⚠️  keycodes.json not found. Using built-in defaults.");
        KeycodeRegistry::new_with_defaults()
    };
    let registry = Arc::new(registry);

    // CHANGED: Pass registry to Search as well
    match cli.command {
        Commands::Search(args) => cmd::search::run(args.clone(), scorer, registry, cli.debug),
        Commands::Validate(args) => cmd::validate::run(args.clone(), &kb_def, scorer, registry),
    }
}
