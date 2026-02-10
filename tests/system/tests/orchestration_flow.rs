#[keyforge_testing_macros::kf_test]
// tests/system/tests/orchestration_flow.rs
use keyforge_model::{types::HandIndex, KeyNode, KeyboardDefinition, Validator};
use keyforge_protocol::JobRequest;
use keyforge_testing::HermeticWorkspace;
use std::sync::Arc;

#[tokio::test]
async fn test_full_orchestration_flow() {
    // 1. Setup Workspace
    let ws = HermeticWorkspace::new()
        .await
        .expect("setup failed")
        .with_default_assets()
        .await
        .expect("assets failed");

    // 2. Mock Hive Request (Simulating CLI upload)
    let mut req = JobRequest::default();
    req.config.definition.geometry.keys.push(
        KeyNode {
<<<<<<< HEAD
            hand: HandIndex::new(0),
=======
            hand: keyforge_model::types::HandIndex::new(0),
>>>>>>> master
            w: 1.0,
            h: 1.0,
            ..Default::default()
        }
        .into(),
    );
    req.config.definition.geometry.home_row = 0;
    req.config
        .definition
        .geometry
        .prime_slots
        .push(keyforge_model::types::KeyIndex::new(0).into());

    assert!(req.validate().is_ok());

    // 3. Prepare Session (Simulating Agent processing)
    let loader = &ws.provider;

    let config_payload = req.config.clone();

    let session = keyforge_compute::SessionBuilder::new(loader)
        .with_keyboard_def(Arc::new(KeyboardDefinition::from_geometry(
            config_payload.to_domain_geometry(),
            "test",
        )))
        .with_corpus(&config_payload.to_domain_corpus_sources())
        .await
        .expect("Failed to load corpus")
        .with_cost_matrix(&config_payload.to_domain_cost_matrix())
        .await
        .expect("Failed to load cost matrix")
        .with_keycodes("keycodes.json")
        .await
        .expect("Failed to load keycodes")
        .with_rubric(keyforge_adapter::conversion::to_domain_rubric(
            &config_payload
                .to_domain_weights()
                .expect("Failed to convert weights"),
        ))
        .build()
        .expect("Failed to prepare session");

    assert_eq!(session.engine.key_count(), 1);

    // 4. Run Optimization (Small steps for speed)
    let mut params_dto = req.config.params.clone();
    params_dto.iterations = 10;

    let model_params = keyforge_model::config::SearchParams {
        params: {
            let mut p = std::collections::HashMap::new();
            p.insert("search_steps".into(), params_dto.iterations as f32);
            p
        },
        ..Default::default()
    };

    let search_config = keyforge_adapter::conversion::to_domain_config(&model_params, 42);
    let engine = session.engine.clone();

    let result = keyforge_compute::optimize_with_engine(
        &engine,
        &search_config,
        keyforge_evolution::NoOpCallback,
        None,
        None,
    )
    .expect("Optimization failed");

<<<<<<< HEAD
    assert!(result.score >= 0.0);
}
=======
    assert!(result.score >= keyforge_model::types::Score::ZERO);
}
>>>>>>> master
