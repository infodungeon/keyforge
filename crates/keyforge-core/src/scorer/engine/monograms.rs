use crate::consts::{KEY_CODE_RANGE, KEY_NOT_FOUND_U8, TIER_COUNT};
use crate::scorer::physics::get_reach_cost;
use crate::scorer::{ScoreDetails, Scorer};

#[inline(always)]
pub fn score_monograms(
    scorer: &Scorer,
    pos_map: &[u8; KEY_CODE_RANGE],
    score: &mut f32,
    left_load: &mut f32,
    total_freq: &mut f32,
) {
    let keys = &scorer.geometry.keys;
    let tiers = &scorer.tier_penalty_matrix;

    for &c_idx in &scorer.active_chars {
        debug_assert!(c_idx < KEY_CODE_RANGE);
        let p = unsafe { *pos_map.get_unchecked(c_idx) };
        if p == KEY_NOT_FOUND_U8 {
            continue;
        }

        let p_idx = p as usize;
        if p_idx >= scorer.key_count {
            continue;
        }

        let freq = unsafe { *scorer.char_freqs.get_unchecked(c_idx) };
        *total_freq += freq;

        if unsafe { keys.get_unchecked(p_idx).hand } == 0 {
            *left_load += freq;
        }

        let char_tier = unsafe { *scorer.char_tier_map.get_unchecked(c_idx) } as usize;
        let slot_tier = unsafe { *scorer.slot_tier_map.get_unchecked(p_idx) } as usize;

        debug_assert!(char_tier < TIER_COUNT);
        debug_assert!(slot_tier < TIER_COUNT);

        unsafe {
            *score += *tiers.get_unchecked(char_tier).get_unchecked(slot_tier) * freq;
            *score += *scorer.slot_monogram_costs.get_unchecked(p_idx) * freq;
        }
    }
}

pub fn accumulate_details(scorer: &Scorer, pos_map: &[u8; KEY_CODE_RANGE], d: &mut ScoreDetails) {
    for &i in &scorer.active_chars {
        let p = pos_map[i];
        let freq = scorer.char_freqs[i];

        if freq > 0.0 {
            d.total_chars += freq;
            if p != KEY_NOT_FOUND_U8 {
                let p_idx = p as usize;
                if p_idx >= scorer.key_count {
                    continue;
                }

                let info = &scorer.geometry.keys[p_idx];
                if info.finger == 4 && info.row != scorer.geometry.home_row {
                    d.stat_pinky_reach += freq;
                }

                d.tier_penalty += scorer.tier_penalty_matrix[scorer.char_tier_map[i] as usize]
                    [scorer.slot_tier_map[p_idx] as usize]
                    * freq;

                if info.is_stretch {
                    d.stat_mono_stretch += freq;
                    d.mech_mono_stretch += scorer.weights.penalty_monogram_stretch * freq;
                }

                d.finger_use += (scorer.finger_scales[info.finger as usize]
                    * scorer.weights.weight_finger_effort)
                    * freq;

                d.geo_dist += get_reach_cost(
                    &scorer.geometry,
                    p_idx,
                    scorer.weights.weight_lateral_travel,
                    scorer.weights.weight_vertical_travel,
                ) * freq;
            }
        }
    }
}

pub fn accumulate_key_costs(scorer: &Scorer, pos_map: &[u8; KEY_CODE_RANGE], costs: &mut [f32]) {
    for &c_idx in &scorer.active_chars {
        let p = pos_map[c_idx];
        if p != KEY_NOT_FOUND_U8 {
            let p_idx = p as usize;
            if p_idx >= scorer.key_count {
                continue;
            }
            let freq = scorer.char_freqs[c_idx];

            costs[p_idx] += scorer.slot_monogram_costs[p_idx] * freq;

            let char_tier = scorer.char_tier_map[c_idx] as usize;
            let slot_tier = scorer.slot_tier_map[p_idx] as usize;
            costs[p_idx] += scorer.tier_penalty_matrix[char_tier][slot_tier] * freq;
        }
    }
}
