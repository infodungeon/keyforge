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
use keyforge_protocol::Validator;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn test_search_params_invariants(
        epochs in 0usize..2_000_000,
        steps in 0usize..10_000_000,
        temp_min in -10.0f32..100.0,
        temp_max in -10.0f32..3000.0,
    ) {
        let mut p = SearchParams::default();
        p.search_epochs = epochs;
        p.search_steps = steps;
        p.temp_min = temp_min;
        p.temp_max = temp_max;

        let res = p.validate();

        // INVARIANT: Invalid ranges must be rejected
        if epochs == 0 || epochs > 1_000_000 {
            prop_assert!(res.is_err());
        }
        if steps == 0 || steps > 5_000_000 {
            prop_assert!(res.is_err());
        }
        if temp_min < 0.0001 || temp_max > 1000.0 || temp_min >= temp_max {
            prop_assert!(res.is_err());
        }
    }

    #[test]
    fn test_scoring_weights_finite_check(
        val in any::<f32>()
    ) {
        let mut w = ScoringWeights::default();
        // Inject the random float into a sensitive field
        w.penalty_sfb_base = val;

        let res = w.validate();

        // INVARIANT: Non-finite or negative values must be rejected
        if !val.is_finite() || val < 0.0 {
            prop_assert!(res.is_err());
        }
        // INVARIANT: Overflow protection
        if val > 100_000_000.0 {
            prop_assert!(res.is_err());
        }
    }
}
