use anyhow::Result;
use std::path::PathBuf;
use tracing::info;
use keyforge_infra::FsProvider;
use keyforge_runner::{RunnerOptions, OptimizationRunner};

use crate::models::AgentConfig;
use crate::config_loader::read_job_config;

pub async fn run(mut config: AgentConfig, job_file: PathBuf, layout: String, timeout: Option<u64>) -> Result<()> {
    info!("Scoring layout: '{}'", layout);

    let job = read_job_config(&job_file).await?;
    
    if let Some(t) = timeout {
        config.compute.job_timeout_sec = t;
    }

    let loader = FsProvider::new(config.data_dir.clone());
    let options = RunnerOptions {
        keycodes_file: config.compute.keycodes_file.clone(),
        ..Default::default()
    };

    let session = OptimizationRunner::prepare_session(
        &loader, &job, &options
    ).await?;

    // Try to resolve as layout name from definition first, then parse as raw string
    let layout_parsed = if let Some(layout_str) = job.definition.layouts.get(&layout) {
        keyforge_adapter::conversion::parse_layout_string(
            layout_str, 
            session.engine.key_count(), 
            &session.registry
        ).map_err(|e| anyhow::anyhow!("Invalid layout in definition: {}", e))?
    } else {
        keyforge_adapter::conversion::parse_layout_string(
            &layout, 
            session.engine.key_count(), 
            &session.registry
        ).map_err(|e| anyhow::anyhow!("Invalid layout string: {}", e))?
    };
    
    let report = session.engine.analyze(&layout_parsed);
    match report {
        Ok(r) => println!("{}", serde_json::to_string_pretty(&r)?),
        Err(e) => return Err(anyhow::anyhow!("Analysis failed: {:?}", e)),
    }
    Ok(())
}
