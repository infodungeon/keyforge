use super::physics::KeyInteraction;
use crate::config::ScoringWeights;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CostCategory {
    None,
    // SFR
    SfrBase,
    SfrBadRow,
    SfrLat,
    SfrWeak,
    // SFB
    SfbBase,
    SfbLat,
    SfbLatWeak,
    SfbDiag,
    SfbLong,
    SfbBot,
    // Other
    Scissor,
    Lateral,
}

#[derive(Debug)]
pub struct CostResult {
    pub penalty_multiplier: f32, // Multiplies distance (SFBs, Scissors)
    pub flow_bonus: f32,         // Subtracts from total (Rolls)
    pub additive_cost: f32,      // Adds to total (SFRs)
    pub category: CostCategory,  // For Reporting
}

pub fn calculate_cost(m: &KeyInteraction, w: &ScoringWeights) -> CostResult {
    let mut res = CostResult {
        penalty_multiplier: 1.0,
        flow_bonus: 0.0,
        additive_cost: 0.0,
        category: CostCategory::None,
    };

    // === FLOW BONUS (Bigrams) ===
    if m.is_roll_in {
        res.flow_bonus += w.bonus_bigram_roll_in;
    } else if m.is_roll_out {
        res.flow_bonus += w.bonus_bigram_roll_out;
    }

    if !m.is_same_hand {
        return res;
    }

    // === SFR (Repeats) ===
    if m.is_repeat {
        if m.is_strong_finger {
            if m.is_home_row {
                res.category = CostCategory::SfrBase; // No Penalty
            } else if m.is_stretch_col {
                res.additive_cost += w.penalty_sfr_lat;
                res.category = CostCategory::SfrLat;
            } else {
                res.additive_cost += w.penalty_sfr_bad_row;
                res.category = CostCategory::SfrBadRow;
            }
        } else {
            res.category = CostCategory::SfrWeak;
            if m.is_home_row {
                res.additive_cost += w.penalty_sfr_weak_finger;
            } else {
                res.additive_cost += w.penalty_sfr_bad_row * 5.0;
            }
        }
        return res; // SFRs do not get SFB penalties
    }

    // === SFB ===
    if m.is_sfb {
        let mut penalty;
        let mut weak_applied = false;

        if m.is_lat_step {
            if m.is_strong_finger {
                penalty = w.penalty_sfb_lateral;
                res.category = CostCategory::SfbLat;
            } else {
                penalty = w.penalty_sfb_lateral_weak;
                res.category = CostCategory::SfbLatWeak;
                weak_applied = true;
            }
        } else if m.is_bot_lat_seq {
            penalty = w.penalty_sfb_bottom;
            res.category = CostCategory::SfbBot;
        } else if m.row_diff >= 2 {
            penalty = w.penalty_sfb_long;
            res.category = CostCategory::SfbLong;
        } else if m.row_diff > 0 && m.col_diff > 0 {
            penalty = w.penalty_sfb_diagonal;
            res.category = CostCategory::SfbDiag;
        } else {
            penalty = w.penalty_sfb_base;
            res.category = CostCategory::SfbBase;
            if m.is_outward {
                penalty += w.penalty_sfb_outward_adder;
            }
        }

        if !m.is_strong_finger && !weak_applied {
            penalty *= w.weight_weak_finger_sfb;
        }

        res.penalty_multiplier = penalty;
        return res;
    }

    // === OTHER ===
    if m.is_scissor {
        res.penalty_multiplier = w.penalty_scissor;
        res.category = CostCategory::Scissor;
    } else if m.is_lateral_stretch {
        res.penalty_multiplier = w.penalty_lateral;
        res.category = CostCategory::Lateral;
    }

    res
}
