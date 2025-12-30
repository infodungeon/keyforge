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
use keyforge_protocol::config::{ScoringWeights, SearchParams};
use keyforge_protocol::geometry::KeyboardGeometry;
use keyforge_protocol::job::JobIdentifier;
use keyforge_protocol::{CostMatrixSource, KeyConstraint};
use keyforge_protocol::types::KeyIndex;

#[test]
fn test_job_identifier_determinism() {
    let geom = KeyboardGeometry::default();
    let weights = ScoringWeights::default();
    let params = SearchParams::default();
    let pins = vec![KeyConstraint {
        index: KeyIndex(0),
        key: "A".into(),
    }];
    let corpus = "default";
    let cost = CostMatrixSource::default();

    let id1 =
        JobIdentifier::try_from_parts(&geom, &weights, &params, &pins, corpus, &cost).unwrap();
    let id2 =
        JobIdentifier::try_from_parts(&geom, &weights, &params, &pins, corpus, &cost).unwrap();

    assert_eq!(id1.hash, id2.hash);
}

#[test]
fn test_job_identifier_sensitivity() {
    let geom = KeyboardGeometry::default();
    let weights = ScoringWeights::default();
    let params = SearchParams::default();
    let pins1 = vec![KeyConstraint {
        index: KeyIndex(0),
        key: "A".into(),
    }];
    let pins2 = vec![KeyConstraint {
        index: KeyIndex(0),
        key: "B".into(),
    }];
    let corpus = "default";
    let cost = CostMatrixSource::default();

    let id1 =
        JobIdentifier::try_from_parts(&geom, &weights, &params, &pins1, corpus, &cost).unwrap();
    let id2 =
        JobIdentifier::try_from_parts(&geom, &weights, &params, &pins2, corpus, &cost).unwrap();

    assert_ne!(id1.hash, id2.hash);
}

#[test]
fn test_job_identifier_cost_matrix_variant() {
    let geom = KeyboardGeometry::default();
    let weights = ScoringWeights::default();
    let params = SearchParams::default();
    let pins = vec![];
    let corpus = "default";

    let cost1 = CostMatrixSource::Predefined("cost.json".into());
    let cost2 = CostMatrixSource::Custom("A,B,1.0".into());

    let id1 =
        JobIdentifier::try_from_parts(&geom, &weights, &params, &pins, corpus, &cost1).unwrap();
    let id2 =
        JobIdentifier::try_from_parts(&geom, &weights, &params, &pins, corpus, &cost2).unwrap();

    assert_ne!(id1.hash, id2.hash);
}
