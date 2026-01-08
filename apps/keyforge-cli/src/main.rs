// apps/keyforge-cli/src/main.rs

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


//! # KeyForge CLI
//!
//! Command-line interface for the KeyForge layout optimization system.
//! This tool allows users to initialize workspaces, fetch assets, and run 
//! local optimizations.

use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use keyforge_compute::Runtime;
use keyforge_infra::resolve_root;
use keyforge_persistence::{Compiler, Project};
use std::error::Error;
use std::path::PathBuf;
use tracing::{error, info, instrument};

mod cli_args;
mod cli_parsers;
mod cmd;
mod error;
use error::CliError;
mod logging;

mod reports;
mod update;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, default_value_t = false)]
    debug: bool,

    #[arg(long, env = "KEYFORGE_DATA_DIR")]
    data_dir: Option<PathBuf>,

    #[arg(long, env = "KEYFORGE_CONFIG")]
    config: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Fetch(cmd::fetch::FetchArgs),
    Completions(cmd::completions::CompletionsArgs),
    Doctor(cmd::doctor::DoctorArgs),
    Fmt(cmd::fmt::FmtArgs),
    Init(cmd::init::InitArgs),
    Search(cmd::search::SearchArgs),
    Validate(cmd::validate::ValidateArgs),
    Benchmark(cmd::benchmark::BenchmarkArgs),
    List(cmd::list::ListArgs),
    Query(cmd::query::QueryArgs),
    Profile(cmd::profile::ProfileArgs),
    Export(cmd::export::ExportArgs),
    Debug(cmd::debug::DebugArgs),
    Update(cmd::update::UpdateArgs),
    Auth(cmd::auth::AuthArgs),
}

#[tokio::main]
async fn main() -> Result<(), CliError> {
    setup_signal_handler();

    if let Err(e) = run_app().await {
        error!("Fatal Error: {}", e);
        return Err(e);
    }
    Ok(())
}

#[instrument]
async fn run_app() -> Result<(), CliError> {
    logging::init_tracing();

    let matches = Cli::command().get_matches();
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());

    // 1. Handle Stateless Commands (No Workspace needed)
    match &cli.command {
        Commands::Init(args) => {
            cmd::init::run(args.clone()).await?;
            return Ok(());
        }
        Commands::Completions(args) => {
            cmd::completions::run(args.clone());
            return Ok(());
        }
        Commands::Auth(args) => {
            cmd::auth::run(args.clone()).await?;
            return Ok(());
        }
        _ => {}
    }

    // Configuration Resolution: CLI > Env > File > Default
    let mut config = keyforge_infra::config::CommonConfig::default();
    if let Some(config_path) = &cli.config {
        match keyforge_infra::config::CommonConfig::from_file(config_path) {
            Ok(file_cfg) => config.merge(file_cfg),
            Err(e) => {
                error!("Failed to load config file {:?}: {}", config_path, e);
                std::process::exit(1);
            }
        }
    }

    // Explicit CLI/Env overrides
    if let Some(d) = cli.data_dir {
        config.data_dir = Some(d);
    }

    let root = resolve_root(config.data_dir)
        .map_err(|e| CliError::Workspace(format!("Workspace Error: {}", e)))?;

    // 2. Handle Stateless Commands (Workspace needed)
    match &cli.command {
        Commands::Doctor(args) => {
            cmd::doctor::run(args.clone(), &root)?;
            return Ok(());
        }
        Commands::Fmt(args) => {
            cmd::fmt::run(args.clone(), &root)?;
            return Ok(());
        }
        Commands::List(args) => {
            cmd::list::run(args.clone(), &root)?;
            return Ok(());
        }
        Commands::Query(args) => {
            cmd::query::run(args.clone(), &root).await?;
            return Ok(());
        }
        Commands::Profile(args) => {
            cmd::profile::run(args.clone())?;
            return Ok(());
        }
        Commands::Export(args) => {
            cmd::export::run(args.clone(), &root)?;
            return Ok(());
        }
        Commands::Fetch(args) => {
            cmd::fetch::run(args.clone(), &root).await?;
            return Ok(());
        }
        Commands::Debug(args) => {
            cmd::debug::run(args.clone(), &root)?;
            return Ok(());
        }
        Commands::Update(args) => {
            cmd::update::run(args.clone()).await?;
            return Ok(());
        }
        _ => {} // Proceed to Runtime commands
    }

    info!("🚀 Initialising KeyForge Core via Compiler…");

    // 2. Handle Runtime Commands (Search, Validate, Benchmark)
    // These require the Physics Engine to be compiled.
    match cli.command {
        Commands::Search(args) => {
            let runtime = build_runtime(&root, &args.shared, args.config.clone()).await?;
            cmd::search::run(args, runtime)?;
        }
        Commands::Validate(args) => {
            let runtime = build_runtime(&root, &args.shared, args.config.clone()).await?;
            cmd::validate::run(args, runtime)?;
        }
        Commands::Benchmark(args) => {
            let runtime = build_runtime(&root, &args.shared, args.config.clone()).await?;
            cmd::benchmark::run(args, runtime)?;
        }
        _ => unreachable!("Stateless commands handled above"),
    }

    Ok(())
}

/// Constructs a Project from CLI args and Compiles it into a Runtime.
async fn build_runtime(
    root: &std::path::Path,
    shared: &cmd::shared::SharedArgs,
    config_args: crate::cli_args::config::ConfigArgs,
) -> Result<Runtime, Box<dyn Error>> {
    // 1. Parse Args
    let corpora = crate::cli_args::parse_corpora(&shared.corpus)?;

    // Resolve Keyboard Path (Handles relative/absolute/workspace paths)
    let kb_path = crate::cli_parsers::resolve_path(&shared.keyboard, Some("keyboards"), root)?;
    let kb_str = kb_path.to_string_lossy().to_string();

    // 2. Load Overrides (Optional Weights File)
    let weights = if let Some(w_input) = &shared.weights {
        let w_path = crate::cli_parsers::resolve_path(w_input, None, root)?;
        let content = keyforge_infra::read_to_string_limited(
            &w_path,
            keyforge_model::constants::MAX_INPUT_FILE_SIZE,
        )?;
        serde_json::from_str(&content)?
    } else {
        use std::convert::TryFrom;
        keyforge_model::config::Config::try_from(config_args.clone())?.weights
    };

    use std::convert::TryFrom;
    let params = keyforge_model::config::Config::try_from(config_args)?.search;

    // 3. Construct Project (The Blueprint)
    let project = Project {
        keyboard: kb_str, // Use resolved absolute path
        corpora,
        weights,
        params,
        constraints: shared.pinned_keys.clone(),
        cost_matrix: keyforge_model::CostMatrixSource::Predefined(shared.cost.clone()),
        seed: None, // Will be set by commands if needed
        ..Default::default()
    };

    // 4. Compile (The Heavy Lift)
    let loader = keyforge_infra::FsProvider::new(root.to_path_buf());
    let compiler = Compiler::new(&loader);

    let runtime = compiler
        .compile(&project)
        .await
        .map_err(|e| CliError::Workspace(format!("Compilation failed: {}", e)))?;

    Ok(runtime)
}

// Global interrupt flag for graceful shutdown
static INTERRUPTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Returns true if the application has received an interrupt signal (e.g., Ctrl+C).
pub fn is_interrupted() -> bool {
    INTERRUPTED.load(std::sync::atomic::Ordering::SeqCst)
}

fn setup_signal_handler() {
    ctrlc::set_handler(|| {
        INTERRUPTED.store(true, std::sync::atomic::Ordering::SeqCst);
        eprintln!("\nShutting down gracefully... (press Ctrl+C again to force quit)");
    })
    .unwrap_or_else(|e| {
        tracing::warn!("Failed to set Ctrl-C handler: {}", e);
    });
}
