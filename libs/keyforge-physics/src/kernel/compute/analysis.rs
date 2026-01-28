use super::flow::calculate_flow_cost;
use super::state::PosMap;
use crate::kernel::{
    types::{FingerIndex, Score, ValidatedLayout},
    EngineContext,
};
use keyforge_model::constants::MAX_REPORTED_VIOLATIONS;
use keyforge_model::{AnalysisReport, KeyCode, MetricId, MetricViolation};

/// Safely converts a u16 character code to a displayable character.
/// Handles invalid Unicode surrogate pairs and control characters.
#[inline]
pub(crate) fn u16_to_char(code: u16) -> String {
    // 1. Common control/special characters
    match code {
        8 => return "⌫".to_string(),  // Backspace
        9 => return "⇥".to_string(),  // Tab
        10 => return "↵".to_string(), // Newline
        32 => return "␣".to_string(), // Space
        _ => {}
    }

    // 2. Try printable Unicode
    if let Some(c) = char::from_u32(u32::from(code)) {
        if !c.is_control() {
            return c.to_string();
        }
        return format!("[0x{code:02X}]");
    }

    // 3. Fallback for invalid Unicode
    format!("[0x{code:04X}]")
}

#[allow(
    clippy::too_many_lines,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_lossless
)]
/// Analyzes a layout and returns a detailed ergonomic report.
///
/// # Errors
/// Returns `PhysicsError::Config` if scratch initialization fails.
pub fn analyze_layout(
    ctx: &EngineContext,
    layout: &ValidatedLayout<'_>,
) -> Result<AnalysisReport, crate::error::PhysicsError> {
    let mut report = AnalysisReport::default();

    super::state::with_scratch(|scratch| {
        let key_count = ctx.key_count;
        let (starts, counts, indices, offsets, used, char_usage, _flat_map) =
            scratch.get_mut_scratch();

        let pm = PosMap::from_scratch(
            layout.as_slice(),
            key_count,
            starts,
            counts,
            indices,
            offsets,
            used,
        );

        let mut heatmap = vec![0.0; key_count];
        let mut penalty_map = vec![0.0; key_count];

        let mut total_load = 0.0;
        let mut left_hand_load = 0.0;
        let mut total_bigrams = 0.0;

        let mut distance = 0.0;
        let mut sfb_total = 0.0;
        let mut sfb_penalty = 0.0;
        let mut scissors = 0.0;
        let mut scissor_penalty = 0.0;
        let mut redirects = 0.0;
        let mut redir_penalty = 0.0;
        let mut rolls = 0.0;
        let mut roll_penalty = 0.0;

        let mut sfbs = Vec::new();
        let mut scissor_violations = Vec::new();
        let mut redirs = Vec::new();

        // 1. Pass 1: Trigrams (Flow ONLY)
        for &(c1, c2, c3, freq) in ctx.all_trigrams.iter() {
            let candidates1 = pm.get(KeyCode(c1));
            let candidates2 = pm.get(KeyCode(c2));
            let candidates3 = pm.get(KeyCode(c3));
            if candidates1.is_empty() || candidates2.is_empty() || candidates3.is_empty() {
                continue;
            }

            let freq_f = freq as f32;
            let mut min_cost_val = Score(i64::MAX);
            let mut best_triplet = (0, 0, 0);

            for &p1 in candidates1 {
                for &p2 in candidates2 {
                    for &p3 in candidates3 {
                        // Score includes flow penalty and travel costs
                        let mut cost =
                            calculate_flow_cost(ctx, p1.as_usize(), p2.as_usize(), p3.as_usize());
                        let idx12 = p1.as_usize() * key_count + p2.as_usize();
                        let idx23 = p2.as_usize() * key_count + p3.as_usize();
                        cost = cost
                            + ctx.geometry.cost_matrix[idx12]
                            + ctx.geometry.cost_matrix[idx23];

                        if cost < min_cost_val {
                            min_cost_val = cost;
                            best_triplet = (p1.as_usize(), p2.as_usize(), p3.as_usize());
                        }
                    }
                }
            }

            if min_cost_val.0 != i64::MAX {
                let (idx1, idx2, idx3) = best_triplet;

                // Flow Effort (Redirects/Rolls) - distributed across triplet
                let flow_cost = calculate_flow_cost(ctx, idx1, idx2, idx3);
                let flow_cost_f32 = flow_cost.to_f32();
                penalty_map[idx1] += flow_cost_f32 * freq_f * 0.33;
                penalty_map[idx2] += flow_cost_f32 * freq_f * 0.33;
                penalty_map[idx3] += flow_cost_f32 * freq_f * 0.33;

                if flow_cost == ctx.penalty_redirect {
                    redirects += freq_f;

                    // Accumulate redirect penalty contribution
                    redir_penalty += flow_cost_f32 * freq_f;

                    redirs.push(MetricViolation {
                        keys: format!("{}{}{}", u16_to_char(c1), u16_to_char(c2), u16_to_char(c3)),
                        score: flow_cost_f32 * freq_f,
                        freq: freq_f,
                    });
                } else if flow_cost < Score::ZERO {
                    rolls += freq_f;

                    // Accumulate roll penalty contribution (negative, so it's a bonus)
                    roll_penalty += flow_cost_f32 * freq_f;
                }
            }
        }

        // 2. Pass 2: Bigrams (ALL TRANSITIONS, DISTANCE, USAGE)
        for &(c1, c2, freq) in ctx.all_bigrams.iter() {
            let candidates1 = pm.get(KeyCode(c1));
            let candidates2 = pm.get(KeyCode(c2));
            if candidates1.is_empty() || candidates2.is_empty() {
                continue;
            }

            let freq_f = freq as f32;
            total_bigrams += freq_f;

            // Choose OPTIMAL key pair by evaluating candidate costs
            let mut min_score = Score(i64::MAX);
            let mut best_pair = (0, 0);

            if candidates1.len() == 1 && candidates2.len() == 1 {
                // Case 1: Single Key - Irrelevant to evaluate choice
                best_pair = (candidates1[0].as_usize(), candidates2[0].as_usize());
            } else {
                // Case 2: Multiple Selection (Duplicated Keys like Space)
                // Pick pair resulting in best score contribution
                for &p1 in candidates1 {
                    for &p2 in candidates2 {
                        let mut cost =
                            ctx.geometry.cost_matrix[p1.as_usize() * key_count + p2.as_usize()];

                        if let Some(&mod_val) = ctx.sequence_modifiers.get(&(c1, c2)) {
                            cost = cost + mod_val;
                        }

                        if cost < min_score {
                            min_score = cost;
                            best_pair = (p1.as_usize(), p2.as_usize());
                        }
                    }
                }
            }

            let (idx1, idx2) = best_pair;

            // --- TRANSITION ACCOUNTING ---
            // Usage (Heatmap) attributed to target character c2
            heatmap[idx2] += freq_f;
            char_usage[c2 as usize] += freq_f;

            // Distance Calculation
            if idx1 == idx2 {
                // Same key: No movement
            } else if ctx.geometry.hands[idx1] == ctx.geometry.hands[idx2] {
                // Same Hand: Euclidean Distance
                distance += ctx.geometry.dist_matrix[idx1 * key_count + idx2] * freq_f;

                // SFB Check (Specific to same-finger move)
                if ctx.geometry.fingers[idx1] == ctx.geometry.fingers[idx2] {
                    sfb_total += freq_f;

                    // Accumulate SFB penalty contribution
                    let cost_val = ctx.geometry.cost_matrix[idx1 * key_count + idx2].to_f32();
                    sfb_penalty += cost_val * freq_f;

                    sfbs.push(MetricViolation {
                        keys: format!("{}{}", u16_to_char(c1), u16_to_char(c2)),
                        score: cost_val * freq_f,
                        freq: freq_f,
                    });
                }
            } else {
                // Different Hand: Movement from home position
                distance += ctx.geometry.key_home_distances[idx2] * freq_f;
            }

            // Scissor Detection
            let r1 = ctx.geometry.rows[idx1];
            let r2 = ctx.geometry.rows[idx2];
            let f1 = ctx.geometry.fingers[idx1];
            let f2 = ctx.geometry.fingers[idx2];
            if ctx.geometry.hands[idx1] == ctx.geometry.hands[idx2]
                && f1.distance(f2) == 1
                && (r1 - r2).abs() >= 2
                && f1 != FingerIndex::THUMB
                && f2 != FingerIndex::THUMB
            {
                scissors += freq_f;

                // Accumulate scissor penalty contribution
                let cost_val = ctx.geometry.cost_matrix[idx1 * key_count + idx2].to_f32();
                scissor_penalty += cost_val * freq_f;

                scissor_violations.push(MetricViolation {
                    keys: format!("{}{}", u16_to_char(c1), u16_to_char(c2)),
                    score: cost_val * freq_f,
                    freq: freq_f,
                });
            }

            // Effort Attribution
            let trans_cost = ctx.geometry.cost_matrix[idx1 * key_count + idx2].to_f32();
            penalty_map[idx1] += trans_cost * freq_f * 0.5;
            penalty_map[idx2] += trans_cost * freq_f * 0.5;
        }

        // 3. Pass 3: Monograms (Base Usage & Remaining Characters)
        for &code in pm.used_keys() {
            let freq = ctx.corpus.char_freqs[code.0 as usize] as f32;
            if freq <= 0.0 {
                continue;
            }

            let candidates = pm.get(code);

            // Monogram Effort (Base Key Cost)
            // Attribute to keys based on their usage heatmap or unique position
            let total_key_usage: f32 = candidates.iter().map(|&p| heatmap[p.as_usize()]).sum();
            if total_key_usage > 0.0 {
                for &p in candidates {
                    let p_idx = p.as_usize();
                    let share = heatmap[p_idx] / total_key_usage;
                    penalty_map[p_idx] += freq * share * ctx.geometry.key_costs[p_idx].to_f32();
                }
            } else {
                // Unused in transitions (e.g. monogram only): use best static key
                let mut min_c = f32::MAX;
                let mut bp = 0;
                for &p in candidates {
                    let c = ctx.geometry.key_costs[p.as_usize()].to_f32();
                    if c < min_c {
                        min_c = c;
                        bp = p.as_usize();
                    }
                }
                heatmap[bp] += freq;
                penalty_map[bp] += freq * ctx.geometry.key_costs[bp].to_f32();
            }
        }

        // Pass 4: Finalize Load Metrics
        for (i, &val) in heatmap.iter().enumerate() {
            total_load += val;
            if ctx.geometry.hands[i].is_left() {
                left_hand_load += val;
            }
        }

        let sort_violations = |v: &mut Vec<MetricViolation>| {
            v.sort_by(|a, b| {
                b.freq
                    .partial_cmp(&a.freq)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            v.truncate(MAX_REPORTED_VIOLATIONS);
        };
        sort_violations(&mut sfbs);
        sort_violations(&mut scissor_violations);
        sort_violations(&mut redirs);

        // Normalization
        let total_freq: u64 = ctx.corpus.char_freqs.iter().sum();
        let mut norm_100k = 1.0;
        let mut norm_pct = 1.0;

        if total_freq > 0 {
            let total_freq_f = total_freq as f64;
            report.travel.travel_per_key = (f64::from(distance) / total_freq_f) as f32;
            norm_100k = (100_000.0 / total_freq_f) as f32;
            norm_pct = (100.0 / total_freq_f) as f32;
        }

        if total_load > 0.0 {
            report.summary.hand_balance = ((left_hand_load / total_load) - 0.5) * -2.0;
        }

        for h in &mut heatmap {
            *h *= norm_pct;
        }
        for p in &mut penalty_map {
            *p *= norm_100k;
        }
        for v in &mut sfbs {
            v.freq *= norm_pct;
            v.score *= norm_100k;
        }
        for v in &mut scissor_violations {
            v.freq *= norm_pct;
            v.score *= norm_100k;
        }
        for v in &mut redirs {
            v.freq *= norm_pct;
            v.score *= norm_100k;
        }

        report.breakdown.violations.insert(MetricId::Sfb, sfbs);
        report
            .breakdown
            .violations
            .insert(MetricId::Scissor, scissor_violations);
        report
            .breakdown
            .violations
            .insert(MetricId::Redirect, redirs);

        report.visualizations.usage = heatmap;
        report.visualizations.effort = penalty_map;

        // Final Score: Sum of context-aware normalized penalties
        report.summary.score = report.visualizations.effort.iter().sum();

        // Normalized metrics
        report.travel.distance = distance * norm_100k;

        report.breakdown.sfb_ratio = if total_bigrams > 0.0 {
            sfb_total / total_bigrams
        } else {
            0.0
        };
        report.breakdown.scissors = scissors * norm_pct;
        report.breakdown.redirects = redirects * norm_pct;
        report.breakdown.rolls = rolls * norm_pct;

        // Populate unified MetricSet
        report
            .breakdown
            .metrics
            .set(MetricId::TravelDistance, report.travel.distance);
        report
            .breakdown
            .metrics
            .set(MetricId::Sfb, sfb_total * norm_pct);
        report
            .breakdown
            .metrics
            .set(MetricId::SfbPenalty, sfb_penalty * norm_100k);
        report
            .breakdown
            .metrics
            .set(MetricId::Scissor, report.breakdown.scissors);
        report
            .breakdown
            .metrics
            .set(MetricId::ScissorPenalty, scissor_penalty * norm_100k);
        report
            .breakdown
            .metrics
            .set(MetricId::Redirect, report.breakdown.redirects);
        report
            .breakdown
            .metrics
            .set(MetricId::RedirectPenalty, redir_penalty * norm_100k);
        report
            .breakdown
            .metrics
            .set(MetricId::RollPenalty, roll_penalty * norm_100k);
        report
            .breakdown
            .metrics
            .set(MetricId::HandBalance, report.summary.hand_balance);

        // Clean up
        scratch.clear_used();
    })?;

    Ok(report)
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;
    use crate::kernel::compiler::Compiler;
    use keyforge_model::types::{ColIndex, FingerIndex, HandIndex, KeyCode, RowIndex};
    use keyforge_model::{Corpus, CostModel, KeyNode, Keyboard, Rubric};
    use std::collections::HashMap;
    use std::sync::Arc;

    #[test]
    fn test_u16_to_char() {
        assert_eq!(u16_to_char(97), "a");
        assert_eq!(u16_to_char(8), "⌫");
        assert_eq!(u16_to_char(9), "⇥");
        assert_eq!(u16_to_char(10), "↵");
        assert_eq!(u16_to_char(32), "␣");
        assert_eq!(u16_to_char(0), "[0x00]");
        assert_eq!(u16_to_char(0xD800), "[0xD800]"); // Invalid surrogate
    }

    #[test]
    fn test_analyze_layout_branches() {
        let mut keys = vec![
            KeyNode {
                index: 0,
                hand: HandIndex::LEFT,
                finger: FingerIndex::INDEX,
                row: RowIndex(0),
                col: ColIndex(0),
                is_home: true,
                ..Default::default()
            },
            KeyNode {
                index: 1,
                hand: HandIndex::LEFT,
                finger: FingerIndex::MIDDLE,
                row: RowIndex(0),
                col: ColIndex(1),
                is_home: true,
                ..Default::default()
            },
            KeyNode {
                index: 2,
                hand: HandIndex::LEFT,
                finger: FingerIndex::RING,
                row: RowIndex(0),
                col: ColIndex(2),
                is_home: true,
                ..Default::default()
            },
        ];
        // Add a duplicate key for space load sharing
        keys.push(KeyNode {
            index: 3,
            hand: HandIndex::LEFT,
            finger: FingerIndex::INDEX,
            row: RowIndex(1),
            col: ColIndex(0),
            is_home: false,
            ..Default::default()
        });

        let kb = Keyboard::new(keys, keyforge_model::types::RowIndex(0), "test".into()).unwrap();
        let mut corpus = Corpus::default();
        let mut freqs = corpus.char_freqs.to_vec();
        freqs[97] = 100; // 'a'
        freqs[98] = 200; // 'b'
        corpus.char_freqs = Arc::from(freqs);
        corpus.bigrams = vec![(97, 98, 50)].into();
        corpus.trigrams = vec![(97, 98, 97, 10)].into(); // Redirect: a -> b -> a (Index -> Middle -> Index)

        let mut fingers = std::collections::HashMap::new();

        let mut base_r0 = keyforge_model::cost_model::RowCosts::new();
        base_r0.insert(RowIndex(0), 1.0);
        let mut base_r1 = keyforge_model::cost_model::RowCosts::new();
        base_r1.insert(RowIndex(1), 2.0);

        let mut index_base = base_r0.clone();
        index_base.extend(base_r1);

        let index_zones = keyforge_model::cost_model::FingerReach {
            base: index_base,
            inner: HashMap::default(),
            outer: HashMap::default(),
        };

        fingers.insert(
            "index".to_string(),
            keyforge_model::cost_model::FingerDefinition::Standard(index_zones),
        );

        let other_zones = keyforge_model::cost_model::FingerReach {
            base: base_r0,
            inner: HashMap::default(),
            outer: HashMap::default(),
        };

        fingers.insert(
            "middle".to_string(),
            keyforge_model::cost_model::FingerDefinition::Standard(other_zones.clone()),
        );
        fingers.insert(
            "ring".to_string(),
            keyforge_model::cost_model::FingerDefinition::Standard(other_zones),
        );

        let mut cm = keyforge_model::cost_model::CostModel::default();
        cm.models.insert(
            "model_a_row_staggered".into(),
            keyforge_model::cost_model::ModelDefinition {
                description: "test".into(),
                static_costs: std::collections::HashMap::from([(
                    "universal_hand".to_string(),
                    keyforge_model::cost_model::HandDefinition { fingers },
                )]),
            },
        );

        let ctx = Compiler::compile(&kb, &corpus, &Rubric::default(), &cm).unwrap();
        let layout_keys = vec![KeyCode(97), KeyCode(98), KeyCode(99), KeyCode(100)];
        let validated = ValidatedLayout::new(&layout_keys, kb.count()).unwrap();

        let report = analyze_layout(&ctx, &validated);
        let report = report.unwrap();
        assert!(report.summary.score > 0.0);
        assert!(report.breakdown.metrics.get(MetricId::Redirect) > 0.0);
    }
}
