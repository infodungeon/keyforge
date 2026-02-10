#![allow(clippy::print_stdout, clippy::print_stderr)]
// apps/keyforge-cli/src/main.rs

use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use indicatif::ProgressBar;
use keyforge_infra::{fs::io::read_to_string_limited, resolve_root};
use keyforge_protocol::JobConfig;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info, instrument};

struct ProgressBarCallback {
    stop_flag: Arc<std::sync::atomic::AtomicBool>,
    pb: ProgressBar,
    start_time: std::time::Instant,
}

impl keyforge_evolution::ProgressCallback for ProgressBarCallback {
    fn on_progress(
        &self,
        epoch: usize,
        score: keyforge_model::Score,
        _layout: &[keyforge_model::KeyCode],
        ips: f32,
    ) -> keyforge_evolution::OptimizationControl {
        if self.stop_flag.load(std::sync::atomic::Ordering::SeqCst) {
            return keyforge_evolution::OptimizationControl::Abort;
        }

        let elapsed = self.start_time.elapsed().as_secs();
        if self.pb.length().is_some() {
            self.pb.set_position(elapsed);
        } else {
            self.pb.tick();
        }

        self.pb.set_message(format!(
            "Epoch {epoch} | Best: {score:.4} | {:.2} MOPS",
            ips / 1_000_000.0
        ));

        keyforge_evolution::OptimizationControl::Continue
    }
}

mod cli_args;
mod cli_parsers;
mod cmd;
pub mod constants;
mod error;
use error::CliError;
mod logging;
mod reports;
// mod update; // REMOVED: Duplicate module declaration. 'update' is already in 'cmd/mod.rs'

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

    if let Commands::Completions(args) = &cli.command {
        cmd::completions::run(args);
        return Ok(());
    }

    let mut config = keyforge_infra::config::CommonConfig::default();
    if let Some(config_path) = &cli.config {
        match keyforge_infra::config::CommonConfig::from_file(config_path) {
            Ok(file_cfg) => config.merge(file_cfg),
            Err(e) => {
                error!(
                    "Failed to load config file {}: {}",
                    config_path.display(),
                    e
                );
                return Err(CliError::Other(format!("Failed to load config: {e}")));
            }
        }
    }
    if let Some(d) = cli.data_dir {
        config.data_dir = Some(d);
    }

    let root = resolve_root(config.data_dir.clone())
        .map_err(|e| CliError::Workspace(format!("Workspace Error: {e}")))?;

    info!("🚀 Initializing Asset Loader...");
    let loader = keyforge_infra::FsProvider::new(root.clone());

    match &cli.command {
        Commands::Init(args) => {
            cmd::init::run(args.clone(), &root).await?;
            return Ok(());
        }
        Commands::Auth(args) => {
            cmd::auth::run(args.clone()).await?;
            return Ok(());
        }
        Commands::Doctor(args) => {
            cmd::doctor::run(args.clone(), &root).await?;
            return Ok(());
        }
        Commands::Fmt(args) => {
            cmd::fmt::run(args, &root)?;
            return Ok(());
        }
        Commands::List(args) => {
            cmd::list::run(args.clone(), &loader).await?;
            return Ok(());
        }
        Commands::Query(args) => {
            cmd::query::run(args.clone(), &root).await?;
            return Ok(());
        }
        Commands::Profile(args) => {
            cmd::profile::run(args)?;
            return Ok(());
        }
        Commands::Export(args) => {
            cmd::export::run(args.clone(), &loader).await?;
            return Ok(());
        }
        Commands::Fetch(args) => {
            cmd::fetch::run(args.clone(), &root).await?;
            return Ok(());
        }
        Commands::Debug(args) => {
            cmd::debug::run(args.clone(), &loader).await?;
            return Ok(());
        }
        Commands::Update(args) => {
            cmd::update::run(args.clone()).await?;
            return Ok(());
        }
        _ => {}
    }

    info!("🚀 Initializing Optimization Runner...");

    match cli.command {
        Commands::Search(args) => {
            cmd::search::run(&args, &loader, &config).await?;
            return Ok(());
        }
        Commands::Validate(args) => {
            cmd::validate::run(&args, &loader).await?;
            return Ok(());
        }
        Commands::Benchmark(args) => {
            cmd::benchmark::run(&args, &loader).await?;
            return Ok(());
        }
        _ => unreachable!("Stateless commands handled above"),
    }
}

async fn build_job_config(
    loader: &keyforge_infra::FsProvider,
    shared: &cmd::shared::SharedArgs,
    config_args: cli_args::config::ConfigArgs,
) -> Result<JobConfig, Box<dyn Error>> {
    use keyforge_adapter::loader::AssetLoader;
    let corpus_list = shared
        .corpus
        .clone()
        .unwrap_or_else(|| vec!["text/en_std".to_string()]);
    let corpora = cli_args::parse_corpora(&corpus_list)?;
    let kb_name = shared
        .keyboard
        .clone()
        .unwrap_or_else(|| "ortho_30".to_string());
    let definition_dto = loader
        .load::<keyforge_protocol::KeyboardDefinitionDto>(&kb_name)
        .await?;

    let weights = if let Some(w_input) = &shared.weights {
        let w_path = cli_parsers::resolve_path(w_input, None, loader.root().as_path())?;
        let content =
            read_to_string_limited(&w_path, keyforge_model::constants::MAX_INPUT_FILE_SIZE)?;
        let weights_dto: keyforge_protocol::ScoringWeightsDto = serde_json::from_str(&content)?;
        weights_dto.into()
    } else {
        keyforge_model::config::Config::try_from(config_args.clone())?.weights
    };

    let params = keyforge_model::config::Config::try_from(config_args)?.search;
    let cost_name = shared
        .cost
        .clone()
        .unwrap_or_else(|| "cost_matrix.json".to_string());

    Ok(keyforge_protocol::JobConfig {
        definition: definition_dto.content.as_ref().clone(),
        weights: weights.into(),
        params: params.into(),
        pinned_keys: shared
            .pinned_keys
            .iter()
            .map(|p| p.clone().into())
            .collect(),
        corpora: corpora.into_iter().map(Into::into).collect(),
        cost_matrix: keyforge_model::config::CostMatrixSource::Predefined {
            id: cost_name,
            hash: None,
        }
        .into(),
        biometrics: vec![].into(),
        parent_job_id: None,
        baseline_score: None,
        parents: vec![].into(),
    })
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
