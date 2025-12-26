export interface KeyNode {
  hand: number;
  finger: number;
  row: number;
  col: number;
  x: number;
  y: number;
  is_stretch?: boolean;
  w?: number;
  h?: number;
  id?: string;
  r?: number;
  rx?: number;
  ry?: number;
}

export interface KeyboardGeometry {
  keys: KeyNode[];
  prime_slots: number[];
  med_slots: number[];
  low_slots: number[];
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

export interface MetricViolation {
  keys: string;
  score: number;
  freq: number;
}

export interface AnalysisReport {
  score: number;
  distance: number;
  sfb_total: number;
  sfb_ratio: number;
  hand_balance: number;
  scissors: number;
  redirects: number;
  rolls: number;
  top_sfbs: MetricViolation[];
  top_scissors: MetricViolation[];
  top_redirs: MetricViolation[];
}

export interface ValidationResult {
  layout_name: string;
  score: AnalysisReport;
  geometry: KeyboardGeometry;
  heatmap: number[];
  penalty_map: number[];
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
  penalty_sfr_weak_finger: number;
  penalty_sfr_bad_row: number;
  penalty_sfr_lat: number;
  penalty_sfb_lateral: number;
  penalty_sfb_lateral_weak: number;
  penalty_sfb_base: number;
  penalty_sfb_outward_adder: number;
  penalty_sfb_diagonal: number;
  penalty_sfb_long: number;
  penalty_sfb_bottom: number;
  weight_weak_finger_sfb: number;

  threshold_sfb_long_row_diff: number;
  threshold_scissor_row_diff: number;

  penalty_scissor: number;
  penalty_ring_pinky: number;
  penalty_lateral: number;
  penalty_monogram_stretch: number;
  penalty_skip: number;
  penalty_redirect: number;
  penalty_hand_run: number;

  bonus_inward_roll: number;
  bonus_bigram_roll_in: number;
  bonus_bigram_roll_out: number;

  penalty_high_in_med: number;
  penalty_high_in_low: number;
  penalty_med_in_prime: number;
  penalty_med_in_low: number;
  penalty_low_in_prime: number;
  penalty_low_in_med: number;

  penalty_imbalance: number;
  max_hand_imbalance: number;

  weight_vertical_travel: number;
  weight_lateral_travel: number;
  weight_finger_effort: number;

  corpus_scale: number;
  default_cost_ms: number;
  loader_trigram_limit: number;

  finger_penalty_scale: string;
  comfortable_scissors: string;

  [key: string]: number | string;
}

export interface JobStatusUpdate {
  active_nodes: number;
  best_score: number;
  best_layout: string;
}

export interface SearchUpdate {
  epoch: number;
  score: number;
  layout: string;
  ips: number;
}

export type AppMode =
  | "analyze"
  | "optimize"
  | "layout"
  | "design"
  | "arena"
  | "test"
  | "settings";

export interface KeycodeDefinition {
  code: number;
  id: string;
  label: string;
  aliases: string[];
}

export interface CorpusSource {
  id: string;
  weight: number;
}

export interface RegisterJobRequest {
  geometry: KeyboardGeometry;
  weights: ScoringWeights;
  params: SearchParams;
  pinned_keys: string;
  corpora: CorpusSource[];
  cost_matrix: string;
}

export interface StartSearchRequest {
  pinned_keys: string;
  search_params: SearchParams;
  weights: ScoringWeights;
}

export interface BiometricSample {
  bigram: string;
  ms: number;
  timestamp: number;
}

export interface UserStatsStore {
  sessions: number;
  total_keystrokes: number;
  biometrics: BiometricSample[];
}
export interface Config {
  search: SearchParams;
  weights: ScoringWeights;
  defs: any;
}

export interface SyncStats {
  downloaded: number;
  merged: number;
  skipped: number;
  errors: string[];
}

export interface SystemHealth {
  cpu_usage: number;
  memory_used: number;
  memory_total: number;
  uptime: number;
  cores: number;
}

export interface CorpusStats {
  name: string;
  size_bytes: number;
  path: string;
}

export interface SwapSuggestion {
  index_a: number;
  index_b: number;
  key_a: string;
  key_b: string;
  score_delta: number;
  improvement_pct: number;
}
