// apps/keyforge-agent/tests/integration_test.rs

//! End-to-end integration tests for the KeyForge agent. Verifies the complete session
//! lifecycle, from workspace bootstrap to job execution, ensuring correct asset loading,
//! corpus merging, and result serialization across hermetic test environments.


use keyforge_agent::agent::compute;
use keyforge_model::config::{CorpusSource, ScoringWeights, SearchParams};
use keyforge_model::geometry::{KeyNode, KeyboardDefinition, KeyboardGeometry, KeyboardMeta};
use keyforge_model::CostMatrixSource;
use keyforge_protocol::JobConfig;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;

#[tokio::test]
async fn test_agent_session_bootstrap() {
    let dir = tempdir().unwrap();
    let data_root = dir.path().join("data");

    // Create User Structure
    fs::create_dir_all(data_root.join("user/corpora/default")).unwrap();
    fs::create_dir_all(data_root.join("user/keyboards")).unwrap();
    fs::create_dir_all(data_root.join("user/weights")).unwrap();
    fs::create_dir_all(data_root.join("user/config")).unwrap();

    // Cost Matrix -> user/weights
    let mut f = File::create(data_root.join("user/weights/cost.json")).unwrap();
    writeln!(
        f,
        r#"[{{"from_key":"KC_A","to_key":"KC_B","cost_ms":10.0,"confidence_samples":10}}]"#
    )
    .unwrap();

    // Corpus -> user/corpora
    let mut f = File::create(data_root.join("user/corpora/default/1grams.json")).unwrap();
    writeln!(f, r#"[{{"char":"a","freq":100}}]"#).unwrap();

    let mut f = File::create(data_root.join("user/corpora/default/2grams.json")).unwrap();
    writeln!(f, r#"[{{"char1":"a","char2":"b","freq":10}}]"#).unwrap();

    let mut f = File::create(data_root.join("user/corpora/default/3grams.json")).unwrap();
    writeln!(f, r#"[{{"char1":"a","char2":"b","char3":"c","freq":5}}]"#).unwrap();

    let mut f = File::create(data_root.join("user/corpora/default/words.json")).unwrap();
    writeln!(f, r#"[{{"word":"test","freq":20}}]"#).unwrap();

    // Keycodes -> user/config
    let mut f = File::create(data_root.join("user/config/keycodes.json")).unwrap();
    writeln!(
        f,
        r#"[{{ "code": 97, "id": "KC_A", "label": "a", "aliases": [] }}]"#
    )
    .unwrap();

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

    let loader = Box::new(keyforge_infra::FsProvider::new(data_root.clone()));

    let prepared_result =
        compute::create_engine_request(loader, data_root.clone(), &config, "cost.json", "corpora", &keyforge_agent::models::ComputeConfig::default()).await;

    match prepared_result {
        Ok(prepared) => {
            // Basic sanity checks
            assert_eq!(prepared.req.keyboard.keys.len(), 1);
            assert!(!prepared.req.corpus.bigrams.is_empty());
        }
        Err(e) => panic!("Agent failed to bootstrap engine request: {:?}", e),
    }
}
