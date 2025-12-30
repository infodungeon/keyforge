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
use keyforge_model::{Rubric, SearchConfig};

#[test]
fn test_rubric_validation() {
    let mut r = Rubric::default();
    assert!(r.validate().is_ok());

    r.trigram_coverage = 1.5;
    assert!(r.validate().is_err());

    r.trigram_coverage = -0.1;
    assert!(r.validate().is_err());

    r.trigram_coverage = 0.5;
    r.trigram_limit = 0;
    assert!(r.validate().is_err());
    
    r.trigram_limit = 100;
    r.sfb_base = -10.0;
    assert!(r.validate().is_err());
}

#[test]
fn test_search_config_validation() {
    let mut c = SearchConfig::default();
    assert!(c.validate().is_ok());

    // Test Steps
    c = SearchConfig::Annealing {
        steps: 0,
        start_temp: 100.0,
        end_temp: 0.01,
        seed: 42,
        patience: 500,
        reheats: 3,
        reheat_factor: 0.5,
    };
    assert!(c.validate().is_err());

    // Test Temp
    c = SearchConfig::Annealing {
        steps: 100,
        start_temp: -1.0,
        end_temp: 0.01,
        seed: 42,
        patience: 500,
        reheats: 3,
        reheat_factor: 0.5,
    };
    assert!(c.validate().is_err());
}
