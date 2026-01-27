use keyforge_model::config::{CorpusSource, CostMatrixSource, ScoringWeights, SearchParams};
#[keyforge_testing_macros::kf_test]
// apps/keyforge-agent/tests/agent_integration.rs
use keyforge_model::geometry::{KeyboardDefinition, KeyboardMeta};
use keyforge_model::types::{ColIndex, FingerIndex, HandIndex, KeyIndex, RowIndex};
use keyforge_model::{KeyNode, KeyboardGeometry};
use keyforge_protocol::JobConfig;
use std::sync::Arc;

#[tokio::test]
#[allow(clippy::unwrap_used)]
async fn test_agent_job_orchestration() {
    // 1. Setup Workspace with default assets (provides cost_matrix.json, en_small.json, etc.)
    let ws = keyforge_testing::HermeticWorkspace::new()
        .await
        .unwrap()
        .with_default_assets()
        .await
        .unwrap();

    let config = JobConfig {
        definition: KeyboardDefinition {
            meta: KeyboardMeta {
                name: "AgentTest".into(),
                ..Default::default()
            },
            geometry: KeyboardGeometry {
                keys: vec![KeyNode {
                    index: 0,
                    x: 0.0,
                    y: 0.0,
                    hand: HandIndex(0),
                    finger: FingerIndex::INDEX,
                    row: RowIndex(0),
                    col: ColIndex(0),
                    ..Default::default()
                }],
                prime_slots: vec![KeyIndex(0)],
                med_slots: vec![],
                low_slots: vec![],
                home_row: keyforge_model::types::RowIndex(0),
            },
            layouts: std::collections::HashMap::default(),
        }
        .into(),
        weights: ScoringWeights::default().into(),
        params: SearchParams::default().into(),
        pinned_keys: vec![].into(),
        corpora: vec![CorpusSource {
            id: "en_small.json".into(),
            weight: 1.0,
            hash: None,
        }
        .into()]
        .into(),
        cost_matrix: CostMatrixSource::Predefined("cost_matrix.json".into()).into(),
        biometrics: vec![].into(),
        parent_job_id: None,
        baseline_score: None,
        parents: vec![].into(),
    };

    let loader = &ws.provider;
    let builder = keyforge_compute::SessionBuilder::new(loader)
        .with_keyboard_def(Arc::new(KeyboardDefinition::from_geometry(
            config.to_domain_geometry(),
            "test",
        )))
        .with_corpus(&config.to_domain_corpus_sources())
        .await
        .unwrap()
        .with_cost_matrix(&config.to_domain_cost_matrix())
        .await
        .unwrap()
        .with_keycodes("keycodes.json")
        .await
        .unwrap()
        .with_rubric(keyforge_adapter::conversion::to_domain_rubric(
            &config.to_domain_weights(),
        ))
        .with_config(keyforge_adapter::conversion::to_domain_config(
            &config.to_domain_params(),
            42,
        ));

    let session = builder.build().unwrap();
    assert!(session.engine.key_count() > 0);
}
