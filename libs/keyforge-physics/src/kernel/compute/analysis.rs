use super::flow::calculate_flow_cost;
use super::state::{PhysicsScratch, PosMap};
use crate::kernel::{
    types::{FingerIndex, Score, ValidatedLayout},
    EngineContext,
};
use keyforge_model::constants::MAX_REPORTED_VIOLATIONS;
use keyforge_model::{AnalysisReport, MetricViolation};
use tracing::instrument;

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
#[instrument(skip_all)]
pub fn analyze_layout(ctx: &EngineContext, layout: &ValidatedLayout<'_>) -> AnalysisReport {
    let mut report = AnalysisReport::default();
    let mut scratch = PhysicsScratch::new();
    let pm = PosMap::from_scratch(
        layout.as_slice(),
        ctx.key_count,
        scratch.starts.as_mut_slice(),
        scratch.counts.as_mut_slice(),
        scratch.indices.as_mut_slice(),
        scratch.current_offsets.as_mut_slice(),
        &mut scratch.used_keys,
    );

    let mut heatmap = vec![0.0; ctx.key_count];
    let mut penalty_map = vec![0.0; ctx.key_count];

    let mut total_load = 0.0;
    let mut left_hand_load = 0.0;
    let mut total_bigrams = 0.0;
    let mut sfbs = Vec::new();
    let mut scissors = Vec::new();
    let mut redirs = Vec::new();

    // 1. Pass 1: Trigrams (Flow ONLY)
    for &(c1, c2, c3, freq) in ctx.all_trigrams.iter() {
        let candidates1 = pm.get(c1 as usize);
        let candidates2 = pm.get(c2 as usize);
        let candidates3 = pm.get(c3 as usize);
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
                    let mut cost = calculate_flow_cost(ctx, p1 as usize, p2 as usize, p3 as usize);
                    let idx12 = (p1 as usize) * ctx.key_count + (p2 as usize);
                    let idx23 = (p2 as usize) * ctx.key_count + (p3 as usize);
                    cost = cost + ctx.geometry.cost_matrix[idx12] + ctx.geometry.cost_matrix[idx23];

                    if cost < min_cost_val {
                        min_cost_val = cost;
                        best_triplet = (p1 as usize, p2 as usize, p3 as usize);
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
                report.redirects += freq_f;

                // Accumulate redirect penalty contribution
                report.redir_penalty += flow_cost_f32 * freq_f;

                redirs.push(MetricViolation {
                    keys: format!("{}{}{}", u16_to_char(c1), u16_to_char(c2), u16_to_char(c3)),
                    score: flow_cost_f32 * freq_f,
                    freq: freq_f,
                });
            } else if flow_cost < Score::ZERO {
                report.rolls += freq_f;

                // Accumulate roll penalty contribution (negative, so it's a bonus)
                report.roll_penalty += flow_cost_f32 * freq_f;
            }
        }
    }

    // 2. Pass 2: Bigrams (ALL TRANSITIONS, DISTANCE, USAGE)
    for &(c1, c2, freq) in ctx.all_bigrams.iter() {
        let candidates1 = pm.get(c1 as usize);
        let candidates2 = pm.get(c2 as usize);
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
            best_pair = (candidates1[0] as usize, candidates2[0] as usize);
        } else {
            // Case 2: Multiple Selection (Duplicated Keys like Space)
            // Pick pair resulting in best score contribution
            for &p1 in candidates1 {
                for &p2 in candidates2 {
                    let mut cost = ctx.geometry.cost_matrix[(p1 as usize) * ctx.key_count + (p2 as usize)];

                    if let Some(&mod_val) = ctx.sequence_modifiers.get(&(c1, c2)) {
                        cost = cost + mod_val;
                    }

                    if cost < min_score {
                        min_score = cost;
                        best_pair = (p1 as usize, p2 as usize);
                    }
                }
            }
        }

        let (idx1, idx2) = best_pair;

        // --- TRANSITION ACCOUNTING ---
        // Usage (Heatmap) attributed to target character c2
        heatmap[idx2] += freq_f;
        scratch.char_usage[c2 as usize] += freq_f;

        // Distance Calculation
        if idx1 == idx2 {
            // Same key: No movement
        } else if ctx.geometry.hands[idx1] == ctx.geometry.hands[idx2] {
            // Same Hand: Euclidean Distance
            report.distance += ctx.geometry.dist_matrix[idx1 * ctx.key_count + idx2] * freq_f;

            // SFB Check (Specific to same-finger move)
            if ctx.geometry.fingers[idx1] == ctx.geometry.fingers[idx2] {
                report.sfb_total += freq_f;

                // Accumulate SFB penalty contribution
                let sfb_cost = ctx.geometry.cost_matrix[idx1 * ctx.key_count + idx2].to_f32();
                report.sfb_penalty += sfb_cost * freq_f;

                sfbs.push(MetricViolation {
                    keys: format!("{}{}", u16_to_char(c1), u16_to_char(c2)),
                    score: sfb_cost * freq_f,
                    freq: freq_f,
                });
            }
        } else {
            // Different Hand: Movement from home position
            report.distance += ctx.geometry.key_home_distances[idx2] * freq_f;
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
            report.scissors += freq_f;

            // Accumulate scissor penalty contribution
            let scissor_cost = ctx.geometry.cost_matrix[idx1 * ctx.key_count + idx2].to_f32();
            report.scissor_penalty += scissor_cost * freq_f;

            scissors.push(MetricViolation {
                keys: format!("{}{}", u16_to_char(c1), u16_to_char(c2)),
                score: scissor_cost * freq_f,
                freq: freq_f,
            });
        }

        // Effort Attribution
        let trans_cost = ctx.geometry.cost_matrix[idx1 * ctx.key_count + idx2].to_f32();
        penalty_map[idx1] += trans_cost * freq_f * 0.5;
        penalty_map[idx2] += trans_cost * freq_f * 0.5;
    }

    // 3. Pass 3: Monograms (Base Usage & Remaining Characters)
    for &code in pm.used_keys {
        let c_val = code as usize;
        #[allow(clippy::cast_precision_loss)]
        let freq = ctx.corpus.char_freqs[c_val] as f32;
        if freq <= 0.0 {
            continue;
        }

        let candidates = pm.get(c_val);

        // Monogram Effort (Base Key Cost)
        // Attribute to keys based on their usage heatmap or unique position
        let total_key_usage: f32 = candidates.iter().map(|&p| heatmap[p as usize]).sum();
        if total_key_usage > 0.0 {
            for &p in candidates {
                let p_idx = p as usize;
                let share = heatmap[p_idx] / total_key_usage;
                penalty_map[p_idx] += freq * share * ctx.geometry.key_costs[p_idx].to_f32();
            }
        } else {
            // Unused in transitions (e.g. monogram only): use best static key
            let mut min_c = f32::MAX;
            let mut bp = 0;
            for &p in candidates {
                let c = ctx.geometry.key_costs[p as usize].to_f32();
                if c < min_c {
                    min_c = c;
                    bp = p as usize;
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
    sort_violations(&mut scissors);
    sort_violations(&mut redirs);

    // Normalization
    let total_freq: u64 = ctx.corpus.char_freqs.iter().sum();
    let mut norm_100k = 1.0;
    let mut norm_pct = 1.0;

    if total_freq > 0 {
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_lossless
        )]
        let total_freq_f = total_freq as f64;
        report.travel_per_key = (f64::from(report.distance) / total_freq_f) as f32;
        norm_100k = (100_000.0 / total_freq_f) as f32;
        norm_pct = (100.0 / total_freq_f) as f32;
    }

    if total_bigrams > 0.0 {
        report.sfb_ratio = report.sfb_total / total_bigrams;
    }
    if total_load > 0.0 {
        report.hand_balance = ((left_hand_load / total_load) - 0.5) * -2.0;
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
    for v in &mut scissors {
        v.freq *= norm_pct;
        v.score *= norm_100k;
    }
    for v in &mut redirs {
        v.freq *= norm_pct;
        v.score *= norm_100k;
    }

    report.top_sfbs = sfbs;
    report.top_scissors = scissors;
    report.top_redirs = redirs;
    report.heatmap = heatmap;
    report.penalty_map = penalty_map;

    // Final Score: Sum of context-aware normalized penalties
    report.score = report.penalty_map.iter().sum();

    // Normalized metrics
    report.distance *= norm_100k;
    report.sfb_total *= norm_pct;
    report.scissors *= norm_pct;
    report.redirects *= norm_pct;
    report.rolls *= norm_pct;

    // Normalize penalty contributions to match score scale
    report.sfb_penalty *= norm_100k;
    report.scissor_penalty *= norm_100k;
    report.redir_penalty *= norm_100k;
    report.roll_penalty *= norm_100k;

    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::compiler::Compiler;
    use keyforge_model::{KeyNode, Keyboard, Corpus, Rubric, CostModel};
    use keyforge_model::types::{KeyCode, HandIndex, FingerIndex, RowIndex, ColIndex};

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
            KeyNode { index: 0, hand: HandIndex::LEFT, finger: FingerIndex::INDEX, row: RowIndex(0), col: ColIndex(0), is_home: true, ..Default::default() },
            KeyNode { index: 1, hand: HandIndex::LEFT, finger: FingerIndex::MIDDLE, row: RowIndex(0), col: ColIndex(1), is_home: true, ..Default::default() },
            KeyNode { index: 2, hand: HandIndex::LEFT, finger: FingerIndex::RING, row: RowIndex(0), col: ColIndex(2), is_home: true, ..Default::default() },
        ];
        // Add a duplicate key for space load sharing
        keys.push(KeyNode { index: 3, hand: HandIndex::LEFT, finger: FingerIndex::INDEX, row: RowIndex(1), col: ColIndex(0), is_home: false, ..Default::default() });
        
        let kb = Keyboard::new(keys, 0, "test".into()).unwrap();
        let mut corpus = Corpus::default();
        corpus.char_freqs[97] = 100; // 'a'
        corpus.char_freqs[98] = 200; // 'b'
        corpus.bigrams.push((97, 98, 50));
        corpus.trigrams.push((97, 98, 97, 10)); // Redirect: a -> b -> a (Index -> Middle -> Index)
        
        let mut cm = CostModel::default();
        let mut fingers = std::collections::HashMap::new();
        let mut base_r0 = std::collections::HashMap::new();
        base_r0.insert("r0".to_string(), 1.0);
        let mut base_r1 = std::collections::HashMap::new();
        base_r1.insert("r1".to_string(), 2.0);
        
        let mut index_zones = std::collections::HashMap::new();
        index_zones.insert("base".to_string(), base_r0.clone());
        index_zones.get_mut("base").unwrap().insert("r1".to_string(), 2.0);
        fingers.insert("index".to_string(), keyforge_model::cost_model::FingerDefinition::Standard(index_zones));
        
        let mut other_zones = std::collections::HashMap::new();
        other_zones.insert("base".to_string(), base_r0);
        fingers.insert("middle".to_string(), keyforge_model::cost_model::FingerDefinition::Standard(other_zones.clone()));
        fingers.insert("ring".to_string(), keyforge_model::cost_model::FingerDefinition::Standard(other_zones));

        cm.models.insert("model_a_row_staggered".into(), keyforge_model::cost_model::ModelDefinition {
            description: "test".into(),
            static_costs: std::collections::HashMap::from([("universal_hand".to_string(), keyforge_model::cost_model::HandDefinition { fingers })]),
        });

        let ctx = Compiler::compile(&kb, &corpus, &Rubric::default(), &cm).unwrap();
        let layout_keys = vec![KeyCode(97), KeyCode(98), KeyCode(99), KeyCode(100)];
        let validated = ValidatedLayout::new(&layout_keys, kb.count()).unwrap();
        
        let report = analyze_layout(&ctx, &validated);
        assert!(report.score > 0.0);
        assert!(report.redirects > 0.0);
    }
}
