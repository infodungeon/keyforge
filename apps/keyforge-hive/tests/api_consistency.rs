use keyforge_protocol::{JobConfig, JobQueueResponse};

#[test]
fn test_queue_response_serialization() {
    let empty = JobQueueResponse {
        job_id: None,
        config: None,
    };
    let json = serde_json::to_string(&empty).unwrap();
    assert_eq!(json, r#"{"job_id":null,"config":null}"#);

    let full = JobQueueResponse {
        job_id: Some("job-123".into()),
        config: Some(JobConfig {
            cost_matrix: keyforge_protocol::CostMatrixSource::Predefined("matrix.json".into()),
            ..JobConfig::from(keyforge_protocol::JobRequest {
                definition: Default::default(),
                weights: Default::default(),
                params: Default::default(),
                pinned_keys: vec![],
                corpora: vec![],
                cost_matrix: keyforge_protocol::CostMatrixSource::Predefined("matrix.json".into()),
                version: 1,
                biometrics: vec![],
                parent_job_id: None,
                baseline_score: None,
                parents: vec![],
            })
        }),
    };

    let json_full = serde_json::to_string(&full).unwrap();
    assert!(json_full.contains("job-123"));
    assert!(json_full.contains("matrix.json"));

    let parsed: JobQueueResponse = serde_json::from_str(&json_full).unwrap();
    assert_eq!(parsed.job_id.unwrap(), "job-123");
}
