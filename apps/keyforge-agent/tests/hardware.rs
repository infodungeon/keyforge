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