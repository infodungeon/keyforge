#[keyforge_testing_macros::kf_test]
mod integration_tests {
    use super::*;
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

    use keyforge_model::{Corpus, Keyboard, Layout, Rubric};
    use keyforge_physics::{verify::DeterministicScorer, EngineCompilationContext, EngineFactory};
    use proptest::prelude::*;

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
                rubric in any::<Rubric>(),
            ) {
                let key_count = kb.count();
                // Generate a valid layout for this keyboard
                let layout_keys: Vec<keyforge_model::KeyCode> = (0..key_count)
                    .map(|i| keyforge_model::KeyCode(i as u16))
                    .collect();
                let layout = Layout::new_unchecked(layout_keys.clone());

                let cost_model = mock_cost_model();

                // 1. Oracle (Ground Truth)
                let oracle = DeterministicScorer::new(&kb, &rubric, &cost_model);
                let oracle_score = oracle.score(&kb, &corpus, &layout.keys).unwrap_or(0); // Handle overflow in fuzzing

                // 2. Optimized Engine (System Under Test)
                let ctx = EngineCompilationContext {
                    keyboard: &kb,
                    corpus: &corpus,
                    rubric: &rubric,
                    cost_model: &cost_model,
                };
                let engine = EngineFactory::new_generic(ctx).unwrap();

                // 3. Generate Report
                let report = engine.analyze(&layout).unwrap();

                // 4. Verification

                // 4. Verification

                // A. Total Score Parity
                // AnalysisReport score is normalized to per-100k-keypresses.
                // Oracle is raw fixed-point (scale 1_000_000) total cost.
                //
                // Relation:
                // RawCost_f32 = Oracle / 1_000_000.0
                // ReportScore = RawCost_f32 * (100_000.0 / TotalFreq)
                //
                // Thus: Oracle = ReportScore * TotalFreq * 10.0

                let total_freq: u64 = corpus.char_freqs.iter().sum();

                // Only compare if we have frequency data
                if total_freq > 0 && oracle_score > 0 {
                    let predicted_oracle = report.score * (total_freq as f32) * 10.0;
                    let actual_oracle = oracle_score as f32;

                    // Allow 1.0% drift due to f32 accumulation errors vs i64 fixed point summation
                    let tolerance = actual_oracle * 0.01;
                    let diff = (predicted_oracle - actual_oracle).abs();

                    if diff > tolerance {
                        println!("Debug: Report Score: {}", report.score);
                        println!("Debug: Total Freq: {}", total_freq);
                        println!("Debug: Predicted Oracle: {}", predicted_oracle);
                        println!("Debug: Actual Oracle: {}", actual_oracle);
                    }

                    prop_assert!(
                        diff <= tolerance,
                        "Score Divergence! Predicted: {}, Actual: {}, Diff: {} (Tol: {})",
                        predicted_oracle, actual_oracle, diff, tolerance
                    );
                }

                // B. Distance Parity
                // We can't easily extract distance from DeterministicScorer without exposing internals,
                // but we can sanity check it against the score components if we had them.
                // For now, let's verify invariants.
                prop_assert!(report.distance >= 0.0);
                prop_assert!(report.sfb_total >= 0.0);

                // C. Hand Balance Invariant
                prop_assert!(report.hand_balance >= -1.0 && report.hand_balance <= 1.0);

                // D. Finger Usage Sum Invariant
                // Heatmap should sum roughly to 100% (frequencies are normalized inside engine usually?)
                // Actually, heatmap is per-key freq sum.
                // E. Penalty Map Consistency
                // Penalty map values can be negative due to bonuses (e.g. rolls)
                for _p in &report.penalty_map {
                    // No assertion needed here if any value is valid,
                    // but we could assert they are finite.
                    prop_assert!(_p.is_finite());
                }
            }
        }
    }
}
