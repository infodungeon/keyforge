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
use super::*;
use keyforge_model::{KeyboardDefinition, ScoringWeights, SearchParams};

#[test]
fn test_version_compatibility() {
    assert!(check_version_compatibility(PROTOCOL_VERSION, PROTOCOL_VERSION).is_ok());
    assert!(check_version_compatibility(0, PROTOCOL_VERSION).is_err());
    assert!(check_version_compatibility(PROTOCOL_VERSION, 0).is_err());
}

#[test]
fn test_job_request_serialization() {
    let req = JobRequest {
        version: PROTOCOL_VERSION,
        definition: KeyboardDefinition::default(),
        weights: ScoringWeights::default(),
        params: SearchParams::default(),
        pinned_keys: vec![],
        corpora: vec![],
        cost_matrix: keyforge_model::CostMatrixSource::default(),
        biometrics: vec![],
        parent_job_id: None,
        baseline_score: None,
        parents: vec![],
    };

    let json = serde_json::to_string(&req).expect("Failed to serialize");
    let deserialized: JobRequest = serde_json::from_str(&json).expect("Failed to deserialize");
    
    assert_eq!(req.version, deserialized.version);
}
