// ===== keyforge/src/main.rs =====
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use keyforge::geometry::KeyboardDefinition;
use keyforge::scorer::Scorer;
use std::path::Path;
use std::process;
use std::sync::Arc;

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
    // 1. Parse Raw Matches (to distinguish user input from defaults)
    let matches = Cli::command().get_matches();

    // 2. Construct CLI struct (populated with defaults)
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());

    println!("\n🚀 Initializing KeyForge Core...");

    // 3. Load Keyboard Definition
    println!("📂 Loading Keyboard: {}", cli.keyboard);
    let kb_def = KeyboardDefinition::load_from_file(&cli.keyboard).unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(1);
    });

    // 4. Extract CLI-provided config AND the specific matches for the subcommand
    // Arguments like --corpus-scale live inside the subcommand's matches, not the root.
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

    // 5. Resolve Weights Strategy: JSON vs CLI
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
                eprintln!(
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
        println!("⚖️  Loading Weights from: {}", path);

        // A. Load JSON weights (this becomes the base)
        let mut file_weights = keyforge::config::ScoringWeights::load_from_file(&path);

        // B. Merge explicit CLI overrides onto the file weights using subcommand matches
        file_weights.merge_from_cli(cli_weights_ref, sub_matches);

        // C. Assign back to config
        config.weights = file_weights;
    } else {
        println!("⚠️  No external weights loaded. Using embedded defaults.");
    }

    // 6. Initialize Scorer
    let scorer_result = Scorer::new(&cli.cost, &cli.ngrams, &kb_def.geometry, config, cli.debug);

    let scorer = match scorer_result {
        Ok(s) => Arc::new(s),
        Err(e) => {
            eprintln!("\n❌ FATAL ERROR INITIALIZING SCORER:");
            eprintln!("   {}", e);
            process::exit(1);
        }
    };

    // 7. Execute
    match cli.command {
        Commands::Search(args) => cmd::search::run(args.clone(), scorer, cli.debug),
        Commands::Validate(args) => cmd::validate::run(args.clone(), &kb_def, scorer),
    }
}
