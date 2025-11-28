use super::physics::KeyInteraction;
use crate::config::ScoringWeights;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CostCategory {
    None,
    // SFR
    SfrBase,
    SfrBadRow, // Used for Strong Bad Row
    SfrLat,
    SfrWeak, // Used for Weak Home/Bad Row
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

pub struct CostResult {
    pub cost: f32,
    pub category: CostCategory,
}

// Removed unused 'dist' parameter
pub fn calculate_interaction_cost(m: &KeyInteraction, w: &ScoringWeights) -> CostResult {
    if m.is_repeat {
        let mut sfr_cost = 0.0;
        let category;

        if m.is_strong_finger {
            if !m.is_home_row {
                sfr_cost += w.penalty_sfr_bad_row;
                if m.is_stretch_col {
                    sfr_cost += w.penalty_sfr_lat - w.penalty_sfr_bad_row;
                    category = CostCategory::SfrLat;
                } else {
                    category = CostCategory::SfrBadRow;
                }
            } else {
                category = CostCategory::SfrBase;
            }
        } else {
            category = CostCategory::SfrWeak;
            if m.is_home_row {
                sfr_cost += w.penalty_sfr_weak_finger;
            } else {
                sfr_cost += w.penalty_sfr_bad_row * 5.0;
            }
        }
        return CostResult {
            cost: sfr_cost,
            category,
        };
    }

    if m.is_sfb {
        if m.is_bot_lat_seq {
            return CostResult {
                cost: w.penalty_sfb_bottom,
                category: CostCategory::SfbBot,
            };
        }
        if m.row_diff >= 2 {
            return CostResult {
                cost: w.penalty_sfb_long,
                category: CostCategory::SfbLong,
            };
        }
        if m.row_diff > 0 && m.col_diff > 0 {
            return CostResult {
                cost: w.penalty_sfb_diagonal,
                category: CostCategory::SfbDiag,
            };
        }
        if m.is_lat_step {
            if m.is_strong_finger {
                return CostResult {
                    cost: w.penalty_sfb_lateral,
                    category: CostCategory::SfbLat,
                };
            } else {
                return CostResult {
                    cost: w.penalty_sfb_lateral_weak,
                    category: CostCategory::SfbLatWeak,
                };
            }
        }

        // Base SFB
        let mut pen = w.penalty_sfb_base;
        if m.is_outward {
            pen += w.penalty_sfb_outward_adder;
        }
        return CostResult {
            cost: pen,
            category: CostCategory::SfbBase,
        };
    }

    if m.is_scissor {
        return CostResult {
            cost: w.penalty_scissor,
            category: CostCategory::Scissor,
        };
    }

    if m.is_lateral_stretch {
        return CostResult {
            cost: w.penalty_lateral,
            category: CostCategory::Lateral,
        };
    }

    CostResult {
        cost: 1.0,
        category: CostCategory::None,
    }
}
