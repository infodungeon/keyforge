// apps/keyforge-agent/tests/network.rs

//! Integration tests for agent network circuit breaking logic. Verifies the reliability
//! of the `CircuitBreaker` in transitioning between Closed, Open, and HalfOpen states
//! based on failure thresholds and recovery timers, ensuring graceful degradation during
//! network instability.


use keyforge_agent::agent::network::CircuitBreaker;
use std::time::Duration;

#[test]
fn test_circuit_breaker_tripping() {
    let mut cb = CircuitBreaker::new(2, 1); // 2 failures, 1s cooldown

    assert!(cb.can_attempt());
    
    cb.record_failure();
    assert!(cb.can_attempt());

    cb.record_failure();
    assert!(!cb.can_attempt(), "Breaker should be tripped");

    // Wait for cooldown
    std::thread::sleep(Duration::from_millis(1100));
    assert!(cb.can_attempt(), "Breaker should allow attempt after cooldown");

    cb.record_success();
    assert!(cb.can_attempt());
}