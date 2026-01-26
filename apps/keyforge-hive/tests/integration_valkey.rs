use keyforge_protocol::NodeTelemetry;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::test]
#[allow(clippy::unwrap_used, clippy::float_cmp)]
async fn test_node_telemetry_serde() {
    let telemetry = NodeTelemetry {
        active_threads: 4,
        cpu_usage: 50.0,
        memory_bytes: 1024,
        job_id: Some("job-123".to_string()),
        ips: 1000.0,
        temp: 0.5,
        current_best: Some(100.0),
        memory_usage: "1024".to_string(),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    };

    let json = serde_json::to_string(&telemetry).unwrap();
    let deserialized: NodeTelemetry = serde_json::from_str(&json).unwrap();
    assert_eq!(telemetry.ips, deserialized.ips);
}
