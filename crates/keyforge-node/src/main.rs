mod calibration;
mod models;
mod nice;
mod worker;

use clap::{Args, Parser, Subcommand};
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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Calibrate => {
            // Calibration always runs full speed (foreground)
            nice::configure_global_thread_pool(false);
            calibration::run_calibration();
        }
        Commands::Work(args) => {
            // Work mode configures environment based on flags
            if args.background {
                nice::set_background_priority();
            }
            nice::configure_global_thread_pool(args.background);

            let node_id = format!(
                "node-{}-{}",
                if args.background { "bg" } else { "fg" },
                Uuid::new_v4().to_string().split('-').next().unwrap()
            );

            worker::run_worker(args.hive, node_id).await;
        }
    }
}
