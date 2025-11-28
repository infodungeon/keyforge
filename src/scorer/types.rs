#[derive(Debug, Default, Clone, Copy)]
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

    // Flow (Weighted Costs & Bonuses)
    pub flow_cost: f32,
    pub flow_redirect: f32,
    pub flow_skip: f32,
    pub flow_roll: f32, // Aggregate Roll Bonus (Legacy/Total)

    // Granular Flow Bonuses (New)
    pub flow_roll_in: f32,  // Bigram Inward Bonus
    pub flow_roll_out: f32, // Bigram Outward Bonus
    pub flow_roll_tri: f32, // Trigram Inward Bonus

    // Heuristics
    pub tier_penalty: f32,
    pub imbalance_penalty: f32,

    // === STATISTICAL COUNTERS (Raw Frequency Sums) ===
    pub total_chars: f32,
    pub total_bigrams: f32,
    pub total_trigrams: f32,

    pub stat_pinky_reach: f32,

    // SFR Stats
    pub stat_sfr: f32,

    // SFB Stats
    pub stat_sfb: f32, // Total SFBs
    pub stat_sfb_base: f32,
    pub stat_sfb_lat: f32,
    pub stat_sfb_lat_weak: f32,
    pub stat_sfb_diag: f32,
    pub stat_sfb_long: f32,
    pub stat_sfb_bot: f32,

    // Non-SFB Stats
    pub stat_lsb: f32, // Total Lateral Stretches (SFB + Non-SFB)
    pub stat_lat: f32, // Non-SFB Lateral
    pub stat_scis: f32,

    // Flow Stats
    pub stat_roll: f32,     // Total Bigram Rolls (In + Out)
    pub stat_roll_in: f32,  // Bigram Inward
    pub stat_roll_out: f32, // Bigram Outward

    pub stat_roll_tri: f32,  // Total Trigram Rolls
    pub stat_roll3_in: f32,  // Trigram Inward
    pub stat_roll3_out: f32, // Trigram Outward

    pub stat_redir: f32,
    pub stat_skip: f32,
}
