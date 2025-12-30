// Copyright (c) 2025 KeyForge Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use keyforge_protocol::config::{CorpusSource, ScoringWeights, SearchParams};
use keyforge_protocol::geometry::KeyboardDefinition;
use keyforge_protocol::{CostMatrixSource, JobConfig, JobRequest};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[test]
fn test_cost_matrix_display() {
    let pre = CostMatrixSource::Predefined("test.json".into());
    assert_eq!(format!("{}", pre), "test.json");

    let cust = CostMatrixSource::Custom("data".into());
    assert_eq!(format!("{}", cust), "<custom_content>");
}

#[test]
fn test_job_config_conversion() {
    let req = JobRequest {
        version: 1,
        definition: KeyboardDefinition::default(),
        weights: ScoringWeights::default(),
        params: SearchParams::default(),
        pinned_keys: vec![],
        corpora: vec![],
        cost_matrix: CostMatrixSource::default(),
        biometrics: vec![],
        parent_job_id: Some("parent".into()),
        baseline_score: Some(100.0),
        parents: vec!["p1".into()],
    };

    let config: JobConfig = req.clone().into();

    assert_eq!(config.parent_job_id, req.parent_job_id);
    assert_eq!(config.baseline_score, req.baseline_score);
    assert_eq!(config.parents, req.parents);
    assert_eq!(config.cost_matrix, req.cost_matrix);
}

#[test]
fn test_corpus_source_hashing() {
    let c1 = CorpusSource {
        id: "a".into(),
        weight: 1.0,
        hash: None,
    };
    let c2 = CorpusSource {
        id: "a".into(),
        weight: 1.0,
        hash: None,
    };
    let c3 = CorpusSource {
        id: "a".into(),
        weight: 0.5,
        hash: None,
    };

    fn calculate_hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    assert_eq!(calculate_hash(&c1), calculate_hash(&c2));
    assert_ne!(calculate_hash(&c1), calculate_hash(&c3));
}
