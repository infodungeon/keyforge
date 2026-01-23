use anyhow::Result;
use keyforge_compute::SessionBuilder;
use keyforge_infra::FsProvider;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

use crate::config_loader::read_job_config;
use crate::models::AgentConfig;

/// Scores a specific layout configuration.
///
/// # Errors
///
/// Returns an error if the job or layout is invalid, or if scoring fails.
pub async fn run(
    mut config: AgentConfig,
    job_file: PathBuf,
    layout: String,
    timeout: Option<u64>,
) -> Result<()> {
    info!("Scoring layout: '{}'", layout);

    let job = read_job_config(&job_file).await?;

    if let Some(t) = timeout {
        config.compute.job_timeout_sec = t;
    }

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

    // Try to resolve as layout name from definition first, then parse as raw string
    let layout_parsed = if let Some(layout_str) = job.definition.layouts.get(&layout) {
        keyforge_adapter::conversion::parse_layout_string(
            layout_str,
            session.engine.key_count(),
            &session.registry,
        )
        .map_err(|e| anyhow::anyhow!("Invalid layout in definition: {e}"))?
    } else {
        keyforge_adapter::conversion::parse_layout_string(
            &layout,
            session.engine.key_count(),
            &session.registry,
        )
        .map_err(|e| anyhow::anyhow!("Invalid layout string: {e}"))?
    };

    let report = session.engine.analyze(&layout_parsed);
    match report {
        Ok(r) => println!("{}", serde_json::to_string_pretty(&r)?),
        Err(e) => return Err(anyhow::anyhow!("Analysis failed: {e:?}")),
    }
    Ok(())
}
