export interface KeyNode {
  hand: number;
  finger: number;
  row: number;
  col: number;
  x: number;
  y: number;
  is_stretch?: boolean;
  // NEW: Added for custom/full-size geometries
  w?: number;
  h?: number;
  id?: string;
}

export interface KeyboardGeometry {
  keys: KeyNode[];
  home_row: number;
}

export interface KeyboardMeta {
  name: string;
  author: string;
  version: string;
  notes: string;
  type: string;
}

export interface KeyboardDefinition {
  meta: KeyboardMeta;
  geometry: KeyboardGeometry;
  layouts: Record<string, string>;
}

export interface ScoreDetails {
  layoutScore: number;
  userScore: number;
  geoDist: number;
  userDist: number;
  fingerUse: number;

  // Mechanics
  mechSfr: number;
  mechSfb: number;
  mechSfbLat: number;
  mechSfbLatWeak: number;
  mechSfbDiag: number;
  mechSfbLong: number;
  mechSfbBot: number;
  mechLat: number;
  mechScis: number;
  mechMonoStretch: number;

  // Flow
  flowCost: number;
  flowRedirect: number;
  flowSkip: number;
  flowRoll: number;
  flowRollIn: number;
  flowRollOut: number;
  flowRollTri: number;

  // Penalties
  tierPenalty: number;
  imbalancePenalty: number;

  // Totals
  totalChars: number;
  totalBigrams: number;
  totalTrigrams: number;

  // Statistics
  statPinkyReach: number;
  statMonoStretch: number;
  statSfr: number;
  statSfb: number;
  statSfbBase: number;
  statSfbLat: number;
  statSfbLatWeak: number;
  statSfbDiag: number;
  statSfbLong: number;
  statSfbBot: number;
  statLsb: number;
  statLat: number;
  statScis: number;
  statRoll: number;
  statRollIn: number;
  statRollOut: number;
  statRollTri: number;
  statRoll3In: number;
  statRoll3Out: number;
  statRedir: number;
  statSkip: number;
}

export interface ValidationResult {
  layoutName: string;
  score: ScoreDetails;
  geometry: KeyboardGeometry;
  heatmap: number[];
}

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
  [key: string]: number;
}

export interface JobStatusUpdate {
  active_nodes: number;
  best_score: number;
  best_layout: string;
}

export type AppMode = 'analyze' | 'optimize' | 'layout' | 'design' | 'arena' | 'test' | 'settings';

export interface KeycodeDefinition {
  code: number;
  id: string;
  label: string;
  aliases: string[];
}