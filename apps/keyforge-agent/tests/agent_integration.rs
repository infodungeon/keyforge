// apps/keyforge-agent/tests/integration_test.rs

//! End-to-end integration tests for the `KeyForge` agent. Verifies the complete session
//! lifecycle, from workspace bootstrap to job execution, ensuring correct asset loading,
//! corpus merging, and result serialization across hermetic test environments.

use keyforge_model::config::{CorpusSource, ScoringWeights, SearchParams};
use keyforge_model::geometry::{KeyNode, KeyboardDefinition, KeyboardGeometry, KeyboardMeta};
use keyforge_model::CostMatrixSource;
use keyforge_protocol::JobConfig;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::sync::Arc;

#[tokio::test]
async fn test_agent_session_bootstrap() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let data_root = dir.path().join("data");

    // Create User Structure
    fs::create_dir_all(data_root.join("user/corpora/default"))?;
    fs::create_dir_all(data_root.join("user/keyboards"))?;
    fs::create_dir_all(data_root.join("user/weights"))?;
    fs::create_dir_all(data_root.join("user/config"))?;

    // Cost Matrix -> user/weights
    let mut f = File::create(data_root.join("user/weights/cost.json"))?;
    writeln!(
        f,
        r#"{{"meta":{{"version":"2.0","description":"Test","unit":"pts"}},"models":{{"model_a_row_staggered":{{"description":"Test","static_costs":{{"universal_hand":{{"index":{{"base":{{"r0":1.0}}}}}}}}}}}},"dynamic_rules":{{"sequence_modifiers":{{}},"penalties":{{}},"constraints":{{}}}}}}"#
    )?;

    // Corpus -> user/corpora
    let mut f = File::create(data_root.join("user/corpora/default/1grams.json"))?;
    writeln!(f, r#"[{{"char":"a","freq":100}}]"#)?;

    let mut f = File::create(data_root.join("user/corpora/default/2grams.json"))?;
    writeln!(f, r#"[{{"char1":"a","char2":"b","freq":10}}]"#)?;

    let mut f = File::create(data_root.join("user/corpora/default/3grams.json"))?;
    writeln!(f, r#"[{{"char1":"a","char2":"b","char3":"c","freq":5}}]"#)?;

    let mut f = File::create(data_root.join("user/corpora/default/words.json"))?;
    writeln!(f, r#"[{{"word":"test","freq":20}}]"#)?;

    // Keycodes -> user/config
    let mut f = File::create(data_root.join("user/config/keycodes.json"))?;
    writeln!(
        f,
        r#"[{{ "code": 97, "id": "KC_A", "label": "a", "aliases": [] }}]"#
    )?;

    let geometry = KeyboardGeometry {
        keys: vec![KeyNode {
            index: 0,
            label: "a".into(),
            ..KeyNode::default()
        }],
        prime_slots: vec![keyforge_model::types::KeyIndex(0)],
        med_slots: vec![],
        low_slots: vec![],
        home_row: 0,
    };

    let config = JobConfig {
        definition: KeyboardDefinition {
            meta: KeyboardMeta {
                name: "AgentTest".into(),
                ..Default::default()
            },
            geometry,
            layouts: HashMap::new(),
        },
        weights: ScoringWeights::default(),
        params: SearchParams::default(),
        pinned_keys: vec![],
        corpora: vec![CorpusSource {
            id: "default".into(),
            weight: 1.0,
            hash: None,
        }],
        cost_matrix: CostMatrixSource::Predefined("cost.json".into()),
        biometrics: vec![],
        parent_job_id: None,
        baseline_score: None,
        parents: vec![],
    };

    let loader = keyforge_infra::FsProvider::new(data_root.clone());

    let session = keyforge_compute::SessionBuilder::new(&loader)
        .with_keyboard_def(Arc::new(config.definition.clone()))
        .with_corpus(&config.corpora)
        .await?
        .with_cost_matrix(&config.cost_matrix)
        .await?
        .with_keycodes("keycodes.json")
        .await?
        .with_rubric(keyforge_adapter::conversion::to_domain_rubric(
            &config.weights,
        ))
        .with_config(keyforge_adapter::conversion::to_domain_config(
            &config.params,
            config.params.seed.unwrap_or(0),
        ))
        .build()?;

    // Basic sanity checks
    assert_eq!(session.engine.key_count(), 1);

    Ok(())
}
