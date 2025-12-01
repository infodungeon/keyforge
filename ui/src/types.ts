export interface KeyNode {
  hand: number;
  finger: number;
  row: number;
  col: number;
  x: number;
  y: number;
}

export interface KeyboardGeometry {
  keys: KeyNode[];
}

export interface ScoreDetails {
  layoutScore: number;
  flowCost: number;
  totalBigrams: number;
  totalTrigrams: number;
  totalChars: number;

  // Stats
  statSfbBase: number;
  statSfbLat: number;
  statSfbLatWeak: number;
  statSfbDiag: number;
  statSfbLong: number;
  statSfbBot: number;
  statLsb: number;
  statScis: number;
  statPinkyReach: number;
  statRoll: number;
  statRollTri: number;
  statRedir: number;
}

export interface ValidationResult {
  layoutName: string;
  score: ScoreDetails;
  geometry: KeyboardGeometry;
  heatmap: number[];
}

// NEW: Search Configuration
export interface SearchParams {
  search_epochs: number;
  search_steps: number;
  search_patience: number;
  search_patience_threshold: number;
  temp_min: number;
  temp_max: number;
  opt_limit_fast: number;
  opt_limit_slow: number;
}

// NEW: Weights Configuration
export interface ScoringWeights {
  penalty_sfb_base: number;
  penalty_sfb_lateral: number;
  penalty_sfb_lateral_weak: number;
  penalty_sfb_diagonal: number;
  penalty_sfb_long: number;
  penalty_sfb_bottom: number;
  penalty_sfr_bad_row: number;
  penalty_sfr_weak_finger: number;
  penalty_scissor: number;
  penalty_lateral: number;
  penalty_redirect: number;
  penalty_skip: number;
  penalty_hand_run: number;
  bonus_inward_roll: number;
  bonus_bigram_roll_in: number;
  bonus_bigram_roll_out: number;
  penalty_imbalance: number;

  threshold_sfb_long_row_diff: number;
  threshold_scissor_row_diff: number;

  // Added defaults for fields that might be missing in older configs
  [key: string]: number;
}

export interface StartSearchRequest {
  pinned_keys: string;
  search_params: SearchParams;
  weights: ScoringWeights;
}