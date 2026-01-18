use anyhow::Result;
use std::path::PathBuf;
use keyforge_infra::FsProvider;
use keyforge_runner::{RunnerOptions, OptimizationRunner};
use keyforge_model::{Layout, KeyCode};

use crate::models::AgentConfig;
use crate::config_loader::read_job_config;

pub async fn run(config: AgentConfig, job_file: PathBuf, iterations: usize) -> Result<()> {
    let job = read_job_config(&job_file).await?;
    
    let loader = FsProvider::new(config.data_dir.clone());
    let options = RunnerOptions {
        keycodes_file: config.compute.keycodes_file.clone(),
        ..Default::default()
    };

    let session = OptimizationRunner::prepare_session(
        &loader, &job, &options
    ).await?;

    let start = std::time::Instant::now();
    let mut score_sum = 0.0;
    
    let engine = session.engine;
    let default_layout = Layout::new_unchecked(vec![KeyCode(0); engine.key_count()]);

    for _ in 0..iterations {
        score_sum += engine.score(&default_layout)?;
    }
    
    let duration = start.elapsed();
    let kops = (iterations as f64 / duration.as_secs_f64()) / 1000.0;
    
    println!("{}", serde_json::json!({
        "iterations": iterations,
        "duration_ms": duration.as_millis(),
        "kops": kops,
        "checksum": score_sum
    }));
    Ok(())
}
