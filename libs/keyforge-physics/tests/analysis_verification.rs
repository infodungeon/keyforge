// libs/keyforge-physics/tests/analysis_verification.rs

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

// # Analysis Report Verification
//
// This test suite ensures that the high-level `AnalysisReport` produced by the
// optimized scoring engines matches the ground truth provided by the `DeterministicScorer`.
//
// Intent: Verify "Oracle Parity" for metric breakdown (Distance, SFBs, etc.).

#[keyforge_testing_macros::kf_test]
mod integration_tests {
    use super::*;
    use keyforge_model::{Corpus, Keyboard, Layout, Rubric};
    use keyforge_physics::{verify::DeterministicScorer, EngineCompilationContext, EngineFactory};
    use proptest::prelude::*;
    use std::sync::Arc;

    #[keyforge_testing_macros::kf_test]
    mod tests {
        use super::*;
        use keyforge_model::testing::mock_cost_model;

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(50))]
            #[test]
            fn test_analysis_report_parity(
                kb in any::<Keyboard>(),
                corpus in any::<Corpus>(),
            ) {
                let rubric = Rubric::default();
                let key_count = kb.count();
                // Generate a valid layout for this keyboard
                let layout_keys: Vec<keyforge_model::KeyCode> = (0..key_count)
                    .map(|i| keyforge_model::KeyCode::new(i as u16))
                    .collect();
                let layout = Layout::new_unchecked(layout_keys.clone());

                let cost_model = mock_cost_model();

<<<<<<< HEAD
                // 1. Oracle (Ground Truth)
                let ctx_comp = EngineCompilationContext {
=======
                // 2. Optimized Engine (System Under Test)
                let ctx = EngineCompilationContext {
>>>>>>> master
                    keyboard: Arc::new(kb.clone()),
                    corpus: Arc::new(corpus.clone()),
                    rubric: Arc::new(rubric.clone()),
                    cost_model: Arc::new(cost_model.clone()),
                    engine_config: keyforge_model::config::EngineConfig::default(),
                };
<<<<<<< HEAD
                let engine_scalar = EngineFactory::new_scalar(&ctx_comp).unwrap();
                let oracle = DeterministicScorer::new(engine_scalar.context().clone());
                let oracle_score = oracle.score(&kb, &corpus, &layout.keys()).unwrap_or(0); // Handle overflow in fuzzing

                // 2. Optimized Engine (System Under Test)
                let engine = EngineFactory::new_generic(&ctx_comp).unwrap();
=======
                let engine = EngineFactory::new_generic(&ctx).unwrap();
                let engine_ctx = engine.context().clone();

                // 1. Oracle (Ground Truth)
                let oracle = DeterministicScorer::new(engine_ctx);
                let oracle_res = oracle.score(&kb, &corpus, &layout.keys());
>>>>>>> master

                // 3. Generate Report
                let report_res = engine.analyze(&layout);

                // If either overflows, we skip this case (fuzzing hit limits)
                prop_assume!(oracle_res.is_ok());
                prop_assume!(report_res.is_ok());

                let oracle_score = oracle_res.unwrap();
                let report = report_res.unwrap();

                // 4. Verification

<<<<<<< HEAD
                // A. Total Score Parity
=======
                // A. Bit-Perfect Raw Score Parity
                // Enforcement: 0% drift (integer exact)
                prop_assert_eq!(
                    report.raw_score.raw(), oracle_score,
                    "Raw Score Divergence! Engine: {}, Oracle: {}",
                    report.raw_score.raw(), oracle_score
                );

                // B. Total Score Parity
>>>>>>> master
                // AnalysisReport score is normalized to per-100k-keypresses.
                //
                // Relation:
                // ReportScore.raw() = deterministic_normalize(RawScore, 100_000, TotalFreq)

                let total_freq: u64 = corpus.char_freqs.iter().sum();

                // Only compare if we have frequency data
                if total_freq > 0 {
                    let tf = total_freq as i128;
                    let product = oracle_score as i128 * 100_000;
                    let half = tf / 2;

                    // Symmetric rounding matching deterministic_normalize (Structural Parity)
                    let expected_score_raw = if product >= 0 {
                        product.checked_div(tf).and_then(|d| {
                            product.checked_rem(tf).and_then(|r| r.checked_add(half)).and_then(|s| s.checked_div(tf)).and_then(|rem| d.checked_add(rem))
                        })
                    } else {
                        product.checked_div(tf).and_then(|d| {
                            product.checked_rem(tf).and_then(|r| r.checked_sub(half)).and_then(|s| s.checked_div(tf)).and_then(|rem| d.checked_add(rem))
                        })
                    };

                    if let Some(expected) = expected_score_raw {
                        let actual_score_raw = report.score.raw();
                        prop_assert_eq!(
                            actual_score_raw, expected as i64,
                            "Normalized Score Divergence! Expected: {}, Actual: {} (TotalFreq: {}, Oracle: {})",
                            expected, actual_score_raw, total_freq, oracle_score
                        );
                    }
                }

                // B. Invariant Checks
                prop_assert!(report.distance >= keyforge_model::Score::ZERO);
                prop_assert!(report.sfb_total >= keyforge_model::Score::ZERO);

                // C. Hand Balance Invariant
                let hi = keyforge_model::Score::from_f32(1.0).unwrap();
                let lo = keyforge_model::Score::from_f32(-1.0).unwrap();
                prop_assert!(report.hand_balance >= lo && report.hand_balance <= hi);

                // D. Penalty Map Consistency
                // Penalty map values can be negative due to bonuses (e.g. rolls)
                for _p in &report.penalty_map {
                    // No assertion needed here if any value is valid
                }
            }
        }
    }
}
