// tests/system/tests/orchestration_flow.rs

use keyforge_model::{types::HandIndex, KeyNode, Validator};
use keyforge_protocol::JobRequest;
use keyforge_testing::HermeticWorkspace;
use std::sync::Arc;

#[tokio::test]
async fn test_full_orchestration_flow() {
    // 1. Setup Workspace
    let ws = HermeticWorkspace::new().with_default_assets();

    // 2. Mock Hive Request (Simulating CLI upload)
    let mut req = JobRequest::default();
    req.config.definition.geometry.keys.push(KeyNode {
        hand: HandIndex(0),
        w: 1.0,
        h: 1.0,
        ..Default::default()
    });
    req.config
        .definition
        .geometry
        .prime_slots
        .push(keyforge_model::KeyIndex(0));

    assert!(req.validate().is_ok());

    // 3. Prepare Session (Simulating Agent processing)
    let loader = &ws.provider;
    let mut options = keyforge_runner::RunnerOptions::default();
    options.keycodes_file = "keycodes.json".to_string();

    let mut config_payload = req.config.clone();
    config_payload.cost_matrix = keyforge_model::CostMatrixSource::Predefined("cost.json".into());

    let session =
        keyforge_runner::OptimizationRunner::prepare_session(loader, &config_payload, &options)
            .await
            .expect("Failed to prepare session");

    assert_eq!(session.engine.key_count(), 1);

    // 4. Run Optimization (Small steps for speed)
    let mut config = req.config.params.clone();
    config.params.insert("search_steps".into(), 10.0);

    let search_config = keyforge_adapter::conversion::to_domain_config(&config, 42);
    let engine = Arc::new(session.engine);

    let result = keyforge_core::optimize_with_engine(
        &engine,
        &search_config,
        keyforge_evolution::NoOpCallback,
        None,
        None,
    )
    .expect("Optimization failed");

    assert!(result.score >= 0.0);
}
