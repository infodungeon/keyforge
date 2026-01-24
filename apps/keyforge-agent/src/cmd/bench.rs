#![allow(clippy::print_stdout)]
use anyhow::Result;
use keyforge_compute::SessionBuilder;
use keyforge_infra::FsProvider;
use keyforge_model::{KeyCode, Layout};
use std::path::PathBuf;
use std::sync::Arc;

use crate::config_loader::read_job_config;
use crate::models::AgentConfig;

/// Runs a micro-benchmark for scoring performance.
///
/// # Errors
///
/// Returns an error if the job configuration cannot be read or scoring fails.
pub async fn run(config: AgentConfig, job_file: PathBuf, iterations: usize) -> Result<()> {
    let job = read_job_config(&job_file).await?;

    let loader = FsProvider::new(config.data_dir.clone());

    let session = SessionBuilder::new(&loader)
        .with_keyboard_def(Arc::new(job.definition.clone()))
        .with_corpus(&job.corpora)
        .await?
        .with_cost_matrix(&job.cost_matrix)
        .await?
        .with_keycodes(&config.compute.keycodes_file)
        .await?
        .with_rubric(keyforge_adapter::conversion::to_domain_rubric(&job.weights))
        .with_config(keyforge_adapter::conversion::to_domain_config(
            &job.params,
            job.params.seed.unwrap_or(0),
        ))
        .build()?;

    let start = std::time::Instant::now();
    let mut score_sum = 0.0;

    let engine = session.engine;
    let default_layout = Layout::new_unchecked(vec![KeyCode(0); engine.key_count()]);

    for _ in 0..iterations {
        score_sum += engine.score(&default_layout)?.to_f32();
    }

    let duration = start.elapsed();
    #[allow(clippy::cast_precision_loss)]
    let kops = (iterations as f64 / duration.as_secs_f64()) / 1000.0;

    println!(
        "{}",
        serde_json::json!({
            "iterations": iterations,
            "duration_ms": duration.as_millis(),
            "kops": kops,
            "checksum": score_sum
        })
    );
    Ok(())
}
