#![allow(clippy::print_stdout)]
// apps/keyforge-agent/src/main.rs

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

//! # `KeyForge` Agent Binary

use anyhow::Result;
use clap::{Parser, Subcommand};
use keyforge_agent::config_loader::load_config_from_standard_paths;
use keyforge_agent::models::{AgentConfig, PartialAgentConfig};
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(long, env = "KEYFORGE_HIVE_URL")]
    hive: Option<String>,

    #[arg(long, env = "KEYFORGE_CORES")]
    cores: Option<usize>,

    #[arg(long, env = "KEYFORGE_DATA_DIR", global = true)]
    data_dir: Option<PathBuf>,

    #[arg(long, env = "KEYFORGE_CONFIG")]
    config: Option<PathBuf>,

    #[arg(long, default_value_t = false)]
    skip_calibration: bool,
}

#[derive(Subcommand, Clone)]
enum Commands {
    /// Start as a long-running worker node connecting to a Hive.
    Worker,
    /// Run a single optimization job defined in a file.
    Run {
        /// Path to the `JobConfig` JSON file.
        job_file: PathBuf,
        /// Maximum time in seconds.
        #[arg(long)]
        timeout: Option<u64>,
    },
    /// Score a specific layout string against a `JobConfig`.
    Score {
        /// Path to the `JobConfig` JSON file.
        job_file: PathBuf,
        /// The layout string to score.
        layout: String,
        /// Maximum time in seconds.
        #[arg(long)]
        timeout: Option<u64>,
    },
    /// Run a physics benchmark using the environment from a `JobConfig`.
    Bench {
        /// Path to the `JobConfig` JSON file.
        job_file: PathBuf,
        /// Number of iterations.
        #[arg(long, default_value_t = keyforge_model::constants::DEFAULT_BENCHMARK_ITERATIONS)]
        iterations: usize,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let mut config = AgentConfig::default();

    if let Some(config_path) = &args.config {
        match PartialAgentConfig::from_file(config_path) {
            Ok(file_cfg) => config.merge(file_cfg),
            Err(e) => {
                eprintln!("Failed to load config file {}: {e}", config_path.display());
                std::process::exit(1);
            }
        }
    } else if let Some(file_cfg) = load_config_from_standard_paths(args.data_dir.as_ref()) {
        config.merge(file_cfg);
    }

    if let Some(h) = args.hive {
        config.hive_url = h;
    }
    if let Some(c) = args.cores {
        config.cores = c;
    }
    if let Some(d) = args.data_dir {
        config.data_dir = d;
    }
    if args.skip_calibration {
        config.calibration.duration_ms = 0;
    }

    let command = args.command.unwrap_or(Commands::Worker);
    let log_mode = match command {
        Commands::Worker => keyforge_agent::logging::LogMode::Standard,
        _ => keyforge_agent::logging::LogMode::JsonStderr,
    };

    keyforge_agent::logging::init_tracing(&config.logging.default_filter, &log_mode);

    match command {
        Commands::Worker => {
            keyforge_agent::cmd::worker::run(config).await?;
        }
        Commands::Run { job_file, timeout } => {
            keyforge_agent::cmd::run::run(config, job_file, timeout).await?;
        }
        Commands::Score {
            job_file,
            layout,
            timeout,
        } => {
            keyforge_agent::cmd::score::run(config, job_file, layout, timeout).await?;
        }
        Commands::Bench {
            job_file,
            iterations,
        } => {
            keyforge_agent::cmd::bench::run(config, job_file, iterations).await?;
        }
    }

    Ok(())
}
