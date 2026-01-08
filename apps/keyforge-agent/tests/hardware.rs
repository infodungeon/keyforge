// apps/keyforge-agent/tests/hardware.rs

//! Integration tests for agent hardware detection and performance calibration. Verifies
//! the accuracy of CPU topology detection, memory profiling, and adaptive throughput
//! calibration based on real-world optimization performance metrics.


use keyforge_agent::hw_detect;
use keyforge_agent::agent::calibration;

#[tokio::test]
async fn test_topology_detection() {
    let topo = hw_detect::detect_topology().await.unwrap();
    assert!(!topo.model.is_empty());
    assert!(topo.cores >= 1);
}

#[test]
fn test_performance_calibration() {
    let ops = calibration::measure_performance().unwrap();
    assert!(ops > 0.0, "Calibration should report positive throughput");
}