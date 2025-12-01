use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreDetails {
    // Top-line Scores
    pub layout_score: f32,
    pub user_score: f32,

    // Raw Distance Components
    pub geo_dist: f32,
    pub user_dist: f32,

    // Effort
    pub finger_use: f32,

    // Mechanics Breakdown (Weighted Costs)
    pub mech_sfr: f32,
    pub mech_sfb: f32,
    pub mech_sfb_lat: f32,
    pub mech_sfb_lat_weak: f32,
    pub mech_sfb_diag: f32,
    pub mech_sfb_long: f32,
    pub mech_sfb_bot: f32,
    pub mech_lat: f32,
    pub mech_scis: f32,

    // NEW: Stretch Cost
    pub mech_mono_stretch: f32,

    // Flow (Weighted Costs & Bonuses)
    pub flow_cost: f32,
    pub flow_redirect: f32,
    pub flow_skip: f32,
    pub flow_roll: f32,

    // Granular Flow Bonuses
    pub flow_roll_in: f32,
    pub flow_roll_out: f32,
    pub flow_roll_tri: f32,

    // Heuristics
    pub tier_penalty: f32,
    pub imbalance_penalty: f32,

    // === STATISTICAL COUNTERS (Raw Frequency Sums) ===
    pub total_chars: f32,
    pub total_bigrams: f32,
    pub total_trigrams: f32,

    pub stat_pinky_reach: f32,

    // NEW: Stretch Frequency
    pub stat_mono_stretch: f32,

    // SFR Stats
    pub stat_sfr: f32,

    // SFB Stats
    pub stat_sfb: f32,
    pub stat_sfb_base: f32,
    pub stat_sfb_lat: f32,
    pub stat_sfb_lat_weak: f32,
    pub stat_sfb_diag: f32,
    pub stat_sfb_long: f32,
    pub stat_sfb_bot: f32,

    // Non-SFB Stats
    pub stat_lsb: f32,
    pub stat_lat: f32,
    pub stat_scis: f32,

    // Flow Stats
    pub stat_roll: f32,
    pub stat_roll_in: f32,
    pub stat_roll_out: f32,

    pub stat_roll_tri: f32,
    pub stat_roll3_in: f32,
    pub stat_roll3_out: f32,

    pub stat_redir: f32,
    pub stat_skip: f32,
}
