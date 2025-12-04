// ===== keyforge/crates/keyforge-node/src/main.rs =====
mod calibration;
mod hw_detect; // FIXED: Added module declaration
mod models;
mod nice;
mod worker;

use clap::{Args, Parser, Subcommand};
use tracing::info;
use uuid::Uuid;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Measures hardware performance to determine batch sizes
    Calibrate,
    /// Connects to the Hive and starts processing jobs
    Work(WorkArgs),
}

#[derive(Args)]
struct WorkArgs {
    /// Hive Server URL
    #[arg(long, default_value = "http://localhost:3000")]
    hive: String,

    /// Run in background mode (Low Priority, Reduced Threads)
    #[arg(long, default_value_t = false)]
    background: bool,
}

fn main() {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Calibrate => {
            nice::configure_global_thread_pool(false);
            calibration::run_calibration();
        }
        Commands::Work(args) => {
            if args.background {
                nice::set_background_priority();
            }

            let physical_cores = num_cpus::get_physical();
            let reserve = if args.background { 2 } else { 1 };
            let compute_threads = if physical_cores > reserve {
                physical_cores - reserve
            } else {
                1
            };

            info!(
                "🧠 Configuring Compute Pool: {} physical threads (System: {})",
                compute_threads, physical_cores
            );

            rayon::ThreadPoolBuilder::new()
                .num_threads(compute_threads)
                .build_global()
                .expect("Failed to initialize global thread pool");

            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .unwrap();

            let node_id = format!(
                "node-{}-{}",
                if args.background { "bg" } else { "fg" },
                Uuid::new_v4().to_string().split('-').next().unwrap()
            );

            rt.block_on(async {
                worker::run_worker(args.hive, node_id).await;
            });
        }
    }
}
