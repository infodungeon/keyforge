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

use keyforge_model::config::ScoringWeights;
use keyforge_model::geometry::{KeyNode, KeyboardGeometry};
use keyforge_model::Validator;
use proptest::prelude::*;

fn weights_strategy() -> impl Strategy<Value = ScoringWeights> {
    (any::<f32>(), any::<f32>(), any::<f32>(), any::<usize>())
        .prop_map(|(sfb, scis, redir, limit)| ScoringWeights {
            penalty_sfb_base: sfb,
            penalty_scissor: scis,
            penalty_redirect: redir,
            loader_trigram_limit: limit,
            ..Default::default()
        })
}

fn geometry_strategy() -> impl Strategy<Value = KeyboardGeometry> {
    proptest::collection::vec(
        (any::<f32>(), any::<f32>(), 0u8..10, 0u8..10),
        0..300,
    )
    .prop_map(|keys| {
        let nodes = keys.into_iter().map(|(w, h, hand, finger)| KeyNode {
            w, h, hand: keyforge_model::types::HandIndex(hand.min(1)), 
            finger: keyforge_model::types::FingerIndex(finger.min(4)), 
            ..Default::default()
        }).collect();
        KeyboardGeometry { keys: nodes, ..Default::default() }
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn fuzz_weights_validation(w in weights_strategy()) {
        let _ = w.validate();
    }

    #[test]
    fn fuzz_geometry_validation(g in geometry_strategy()) {
        let _ = g.validate();
    }

    #[test]
    fn fuzz_json_deserialization(s in "\\PC*") {
        let _ = serde_json::from_str::<keyforge_protocol::JobRequest>(&s);
    }
}