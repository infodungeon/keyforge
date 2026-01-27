use tracing::info;
// apps/keyforge-agent/src/cmd/score.rs

use crate::agent::compute::prepare_assets;
use crate::models::ComputeConfig;
use anyhow::Result;
use keyforge_infra::AssetManager;
use keyforge_model::KeyboardDefinition;
use keyforge_protocol::JobConfig;
use std::sync::Arc;

pub async fn run(
    assets: &AssetManager,
    job: &JobConfig,
    config: &ComputeConfig,
    layout_str: &str,
) -> Result<()> {
    let (_cost_name, _corpus_id) = prepare_assets(assets, job, config).await?;

    let loader = keyforge_infra::FsProvider::new(assets.root().to_path_buf());
    let mut builder = keyforge_compute::SessionBuilder::new(&loader);

    builder = builder.with_keyboard_def(Arc::new(KeyboardDefinition::from_geometry(
        job.to_domain_geometry(),
        "score",
    )));
    builder = builder.with_corpus(&job.to_domain_corpus_sources()).await?;
    builder = builder
        .with_cost_matrix(&job.to_domain_cost_matrix())
        .await?;
    builder = builder.with_keycodes(&config.keycodes_file).await?;

    let builder = builder
        .with_rubric(keyforge_adapter::conversion::to_domain_rubric(
            &job.to_domain_weights(),
        ))
        .with_config(keyforge_adapter::conversion::to_domain_config(
            &job.to_domain_params(),
            job.params.seed.unwrap_or(0),
        ));

    let session = builder.build()?;
    let layout = keyforge_adapter::conversion::parse_layout_string(
        layout_str,
        session.engine.key_count(),
        &session.registry,
    )?;

    let result = session.engine.score(&layout)?;

    #[allow(clippy::cast_precision_loss)]
    let score_f32 = result.0 as f32;

    info!("Score: {:.4}", score_f32 / 1_000_000.0);
    Ok(())
}
