// ===== keyforge/crates/keyforge-cli/src/main.rs =====
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

    // CHANGED: "ngrams" file -> "corpus" directory
    #[arg(global = true, long, default_value = "data/corpora/default")]
    corpus: String,

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
    Benchmark(cmd::benchmark::BenchmarkArgs),
}

fn main() {
    tracing_subscriber::fmt::init();

    let matches = Cli::command().get_matches();
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());

    info!("🚀 Initializing KeyForge Core...");

    info!("📂 Loading Keyboard: {}", cli.keyboard);
    let kb_def = KeyboardDefinition::load_from_file(&cli.keyboard).unwrap_or_else(|e| {
        error!("Failed to load keyboard definition: {}", e);
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
        Commands::Benchmark(args) => (
            args.config.clone(),
            &args.config.weights,
            matches.subcommand_matches("benchmark").unwrap(),
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
        filename.map(|name| format!("data/weights/{}", name))
    };

    if let Some(path) = weights_path_str {
        if Path::new(&path).exists() {
            info!("⚖️  Loading Weights from: {}", path);
            let mut file_weights = keyforge_core::config::ScoringWeights::load_from_file(&path);
            file_weights.merge_from_cli(cli_weights_ref, sub_matches);
            config.weights = file_weights;
        } else {
            warn!(
                "⚠️  Weights file '{}' not found. Using embedded defaults.",
                path
            );
        }
    }

    // Scorer Init now takes directory
    let scorer_res = Scorer::new(
        &cli.cost,
        &cli.corpus, // Directory
        &kb_def.geometry,
        config.clone(),
        cli.debug,
    );

    let scorer = match scorer_res {
        Ok(s) => Arc::new(s),
        Err(e) => {
            error!("\n❌ FATAL ERROR INITIALIZING SCORER:");
            error!("   {}", e);
            process::exit(1);
        }
    };

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
    let registry_arc = Arc::new(registry);

    match cli.command {
        Commands::Search(args) => cmd::search::run(args.clone(), scorer, registry_arc, cli.debug),
        Commands::Validate(args) => cmd::validate::run(args.clone(), &kb_def, scorer, registry_arc),
        Commands::Benchmark(args) => cmd::benchmark::run(args, scorer),
    }
}
