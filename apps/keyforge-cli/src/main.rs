// apps/keyforge-cli/src/main.rs

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


//! # KeyForge CLI
//!
//! Command-line interface for the KeyForge layout optimization system.

use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use keyforge_infra::resolve_root;
use std::path::PathBuf;
use tracing::{error, instrument};

mod cli_args;
mod cli_parsers;
mod cmd;
mod error;
mod constants;
use error::CliError;
mod logging;

mod reports;
// Note: update module is inside cmd/

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

    // Configuration Resolution
    let mut config = keyforge_infra::config::CommonConfig::default();
    if let Some(config_path) = &cli.config {
        match keyforge_infra::config::CommonConfig::from_file(config_path) {
            Ok(file_cfg) => config.merge(file_cfg),
            Err(e) => {
                error!("Failed to load config file {:?}: {}", config_path, e);
                return Err(CliError::Other(format!("Failed to load config: {}", e)));
            }
        }
    }

    if let Some(d) = cli.data_dir {
        config.data_dir = Some(d);
    }

    let root = resolve_root(config.data_dir)
        .map_err(|e| CliError::Workspace(format!("Workspace Error: {}", e)))?;

    // 2. Handle Stateless Commands (Workspace needed)
    match &cli.command {
        Commands::Doctor(args) => {
            cmd::doctor::run(args.clone(), &root).await?;
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
        _ => {} 
    }

    // 3. Sidecar Commands (Delegated to Agent)
    match cli.command {
        Commands::Search(args) => {
            cmd::search::run(args, &root)?;
        }
        Commands::Validate(args) => {
            cmd::validate::run(args, &root)?;
        }
        Commands::Benchmark(args) => {
            cmd::benchmark::run(args, &root)?;
        }
        _ => {} // Handled above
    }

    Ok(())
}

static INTERRUPTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

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
