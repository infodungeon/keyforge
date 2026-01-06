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