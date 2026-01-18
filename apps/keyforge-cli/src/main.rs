// apps/keyforge-cli/src/main.rs

use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use keyforge_infra::resolve_root;
use keyforge_protocol::JobConfig;
use keyforge_model::KeyboardDefinition;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info, instrument};

mod cli_args;
mod cli_parsers;
mod cmd;
mod error;
pub mod constants;
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

    match &cli.command {
        Commands::Init(args) => { cmd::init::run(args.clone()).await?; return Ok(()); }
        Commands::Completions(args) => { cmd::completions::run(args.clone()); return Ok(()); }
        Commands::Auth(args) => { cmd::auth::run(args.clone()).await?; return Ok(()); }
        _ => {}
    }

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
    if let Some(d) = cli.data_dir { config.data_dir = Some(d); }

    let root = resolve_root(config.data_dir)
        .map_err(|e| CliError::Workspace(format!("Workspace Error: {}", e)))?;

    info!("🚀 Initializing Asset Loader...");
    let loader = keyforge_infra::FsProvider::new(root.clone());

    match &cli.command {
        Commands::Doctor(args) => { cmd::doctor::run(args.clone(), &root).await?; return Ok(()); }
        Commands::Fmt(args) => { cmd::fmt::run(args.clone(), &root)?; return Ok(()); }
        Commands::List(args) => { cmd::list::run(args.clone(), &loader).await?; return Ok(()); }
        Commands::Query(args) => { cmd::query::run(args.clone(), &root).await?; return Ok(()); }
        Commands::Profile(args) => { cmd::profile::run(args.clone())?; return Ok(()); }
        Commands::Export(args) => { cmd::export::run(args.clone(), &root)?; return Ok(()); }
        Commands::Fetch(args) => { cmd::fetch::run(args.clone(), &root).await?; return Ok(()); }
        Commands::Debug(args) => { cmd::debug::run(args.clone(), &loader).await?; return Ok(()); }
        Commands::Update(args) => { cmd::update::run(args.clone()).await?; return Ok(()); }
        _ => {} 
    }

    info!("🚀 Initializing Optimization Runner...");

    match cli.command {
        Commands::Search(args) => {
            let options = keyforge_runner::RunnerOptions {
                timeout_sec: args.time.unwrap_or(3600),
                seed: args.seed,
                threads: args.threads,
                keycodes_file: "keycodes.json".into(),
                ..Default::default()
            };
            let job = build_job_config(&loader, &args.shared, args.config.clone()).await?;
            let session = keyforge_runner::OptimizationRunner::prepare_session(&loader, &job, &options).await?;
            
            let stop_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
            
            // Task-cli-028: Setup Progress Bar
            use indicatif::{ProgressBar, ProgressStyle};
            let pb = ProgressBar::new(options.timeout_sec);
            pb.set_style(ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len}s ({eta}) {msg}")
                .unwrap()
                .progress_chars("#>-"));
            pb.set_message("Optimizing layout...");

            struct ProgressBarCallback {
                stop_flag: Arc<std::sync::atomic::AtomicBool>,
                pb: ProgressBar,
                start_time: std::time::Instant,
            }

            impl keyforge_evolution::ProgressCallback for ProgressBarCallback {
                fn on_progress(&self, epoch: usize, score: f32, _layout: &[keyforge_model::KeyCode], ips: f32) -> bool {
                    let elapsed = self.start_time.elapsed().as_secs();
                    self.pb.set_position(elapsed);
                    self.pb.set_message(format!("Epoch {} | Best: {:.4} | {:.0} ips", epoch, score, ips));
                    !self.stop_flag.load(std::sync::atomic::Ordering::SeqCst)
                }
            }

            let callback = ProgressBarCallback { 
                stop_flag: stop_flag.clone(), 
                pb: pb.clone(),
                start_time: std::time::Instant::now()
            };
            
            let result: keyforge_model::OptimizationResult = keyforge_runner::OptimizationRunner::run(
                session, 
                "local-cli".into(), 
                stop_flag, 
                callback, 
                options, 
                &job
            ).await?;
            
            pb.finish_with_message("Optimization complete.");
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Commands::Validate(args) => {
            let options = keyforge_runner::RunnerOptions {
                keycodes_file: "keycodes.json".into(),
                ..Default::default()
            };
            let job = build_job_config(&loader, &args.shared, args.config.clone()).await?;
            let session = keyforge_runner::OptimizationRunner::prepare_session(&loader, &job, &options).await?;
            
            let layout_name = args.layout.as_deref().unwrap_or("default");
            let layout_parsed = if let Some(l_str) = job.definition.layouts.get(layout_name) {
                keyforge_adapter::conversion::parse_layout_string(l_str, session.engine.key_count(), &session.registry)?
            } else {
                keyforge_adapter::conversion::parse_layout_string(layout_name, session.engine.key_count(), &session.registry)?
            };

            let report = session.engine.analyze(&layout_parsed)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Commands::Benchmark(args) => {
            let options = keyforge_runner::RunnerOptions {
                keycodes_file: "keycodes.json".into(),
                ..Default::default()
            };
            let job = build_job_config(&loader, &args.shared, args.config.clone()).await?;
            let session = keyforge_runner::OptimizationRunner::prepare_session(&loader, &job, &options).await?;
            
            let start = std::time::Instant::now();
            let mut score_sum = 0.0;
            let default_layout = keyforge_model::Layout::new_unchecked(vec![keyforge_model::KeyCode(0); session.engine.key_count()]);

            for _ in 0..args.iterations {
                score_sum += session.engine.score(&default_layout)?;
            }
            
            let duration = start.elapsed();
            let kops = (args.iterations as f64 / duration.as_secs_f64()) / 1000.0;
            
            println!("{}", serde_json::json!({
                "iterations": args.iterations,
                "duration_ms": duration.as_millis(),
                "kops": kops,
                "checksum": score_sum
            }));
        }
        _ => unreachable!("Stateless commands handled above"),
    }
    Ok(())
}

async fn build_job_config(
    loader: &keyforge_infra::FsProvider,
    shared: &cmd::shared::SharedArgs,
    config_args: cli_args::config::ConfigArgs,
) -> Result<JobConfig, Box<dyn Error>> {
    use keyforge_core::loader::AssetLoader;
    let corpus_list = shared.corpus.clone().unwrap_or_else(|| vec!["text/en_std".to_string()]);
    let corpora = cli_args::parse_corpora(&corpus_list)?;
    let kb_name = shared.keyboard.clone().unwrap_or_else(|| "ortho_30".to_string());
    let definition = loader.load::<KeyboardDefinition>(&kb_name).await?;

    let weights = if let Some(w_input) = &shared.weights {
        let w_path = cli_parsers::resolve_path(w_input, None, &loader.root)?;
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
    let cost_name = shared.cost.clone().unwrap_or_else(|| "default_costmatrix.json".to_string());

    Ok(JobConfig {
        definition: (*definition).clone(),
        weights,
        params,
        pinned_keys: shared.pinned_keys.clone(),
        corpora,
        cost_matrix: keyforge_model::CostMatrixSource::Predefined(cost_name),
        biometrics: vec![],
        parent_job_id: None,
        baseline_score: None,
        parents: vec![],
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
    }).unwrap_or_else(|e| {
        tracing::warn!("Failed to set Ctrl-C handler: {}", e);
    });
}
