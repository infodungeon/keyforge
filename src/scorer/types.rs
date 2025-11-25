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

    // Mechanics Breakdown
    pub mech_sfr: f32, // Repeats
    pub mech_sfb: f32, // Standard SFB

    // SFB Types
    pub mech_sfb_lat: f32,      // Strong Lateral (Index TG)
    pub mech_sfb_lat_weak: f32, // NEW: Weak Lateral

    pub mech_sfb_diag: f32, // Diagonal SFB
    pub mech_sfb_long: f32, // Long SFB (TB)
    pub mech_sfb_bot: f32,  // Bot-Lat SFB (Claw)

    pub mech_lat: f32,  // Non-SFB Lateral
    pub mech_scis: f32, // Scissor

    // Legacy (Unused in reporting but kept for compatibility)
    pub penalty_geo: f32,
    pub penalty_user: f32,
    pub geo_sfb: f32,
    pub geo_dsfb: f32,
    pub geo_lsfb: f32,
    pub geo_bsfb: f32,
    pub geo_lateral: f32,
    pub geo_scissor: f32,
    pub user_penalty_sfb: f32,
    pub user_penalty_lat: f32,
    pub user_penalty_scis: f32,

    // Flow
    pub flow_cost: f32,
    pub flow_redirect: f32,
    pub flow_skip: f32,
    pub flow_run: f32,
    pub flow_roll: f32,

    // Heuristics
    pub tier_penalty: f32,
    pub imbalance_penalty: f32,
}
