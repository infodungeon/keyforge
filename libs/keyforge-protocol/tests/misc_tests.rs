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

use keyforge_model::{
    CorpusSource, CostMatrixSource, KeyConstraint, KeyboardDefinition, ScoringWeights, SearchParams,
    Validator, KeyNode, KeyIndex
};
use keyforge_protocol::{JobConfig, JobRequest};

#[test]
fn test_job_config_conversion() {
    let req = JobRequest {
        version: 1,
        definition: KeyboardDefinition::default(),
        weights: ScoringWeights::default(),
        params: SearchParams::default(),
        pinned_keys: vec![KeyConstraint {
            index: KeyIndex(0),
            key: "A".into(),
        }],
        corpora: vec![CorpusSource::default()],
        cost_matrix: CostMatrixSource::default(),
        biometrics: vec![],
        parent_job_id: Some("parent".into()),
        baseline_score: Some(100.0),
        parents: vec!["p1".into()],
    };
    let config: JobConfig = req.clone().into();
    assert_eq!(config.pinned_keys.len(), 1);
    assert_eq!(config.parent_job_id, Some("parent".into()));
}

#[test]
fn test_job_request_validation() {
    let mut req = JobRequest {
        version: 1,
        definition: KeyboardDefinition::default(),
        weights: ScoringWeights::default(),
        params: SearchParams::default(),
        pinned_keys: vec![],
        corpora: vec![],
        cost_matrix: CostMatrixSource::default(),
        biometrics: vec![],
        parent_job_id: None,
        baseline_score: None,
        parents: vec![],
    };
    
    // Setup valid minimal geometry
    req.definition.geometry.keys = vec![KeyNode::default()];
    req.definition.geometry.prime_slots = vec![KeyIndex(0)];

    // Test 1: Too many keys
    let original_keys = req.definition.geometry.keys.clone();
    req.definition.geometry.keys = vec![KeyNode::default(); 201];
    assert!(req.validate().is_err(), "Should reject > 200 keys");
    req.definition.geometry.keys = original_keys;

    // Test 2: Pinned key out of bounds
    req.pinned_keys = vec![KeyConstraint {
        index: KeyIndex(5), // Only 1 key exists (index 0)
        key: "A".into(),
    }];
    assert!(req.validate().is_err(), "Should reject out of bounds pin");
    req.pinned_keys.clear();

    // Test 3: Cost Matrix Empty
    req.cost_matrix = CostMatrixSource::Predefined("".into());
    assert!(req.validate().is_err(), "Should reject empty cost matrix name");
}
