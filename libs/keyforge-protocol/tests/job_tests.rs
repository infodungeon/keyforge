use keyforge_protocol::config::{ScoringWeights, SearchParams};
use keyforge_protocol::geometry::KeyboardGeometry;
use keyforge_protocol::job::JobIdentifier;
use keyforge_protocol::protocol::{CostMatrixSource, KeyConstraint};

#[test]
fn test_job_identifier_determinism() {
    let geom = KeyboardGeometry::default();
    let weights = ScoringWeights::default();
    let params = SearchParams::default();
    let pins = vec![KeyConstraint {
        index: 0,
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
        index: 0,
        key: "A".into(),
    }];
    let pins2 = vec![KeyConstraint {
        index: 0,
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
