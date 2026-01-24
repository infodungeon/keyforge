#[keyforge_testing_macros::kf_test]
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
    req.config.definition.geometry.home_row = 0;
    req.config
        .definition
        .geometry
        .prime_slots
        .push(keyforge_model::KeyIndex(0));

    assert!(req.validate().is_ok());

    // 3. Prepare Session (Simulating Agent processing)
    let loader = &ws.provider;

    let mut config_payload = req.config.clone();
    config_payload.cost_matrix = keyforge_model::CostMatrixSource::Predefined("cost.json".into());

    let session = keyforge_compute::SessionBuilder::new(loader)
        .with_keyboard_def(Arc::new(config_payload.definition.clone()))
        .with_corpus(&config_payload.corpora)
        .await
        .expect("Failed to load corpus")
        .with_cost_matrix(&config_payload.cost_matrix)
        .await
        .expect("Failed to load cost matrix")
        .with_keycodes("keycodes.json")
        .await
        .expect("Failed to load keycodes")
        .with_rubric(keyforge_adapter::conversion::to_domain_rubric(
            &config_payload.weights,
        ))
        .build()
        .expect("Failed to prepare session");

    assert_eq!(session.engine.key_count(), 1);

    // 4. Run Optimization (Small steps for speed)
    let mut config = req.config.params.clone();
    config.params.insert("search_steps".into(), 10.0);

    let search_config = keyforge_adapter::conversion::to_domain_config(&config, 42);
    let engine = session.engine.clone();

    let result = keyforge_compute::optimize_with_engine(
        &engine,
        &search_config,
        keyforge_evolution::NoOpCallback,
        None,
        None,
    )
    .expect("Optimization failed");

    assert!(result.score >= 0.0);
}
