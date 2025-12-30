import type {
  KeyNode, KeyboardGeometry, KeyboardMeta, KeyboardDefinition,
  MetricViolation, AnalysisReport, SearchParams, ScoringWeights,
  KeycodeDefinition, CorpusSource, BiometricSample, UserStatsStore,
  Config, SystemMetrics, JobStatus, SwapSuggestion, JobRequest
} from "./types/generated";

export type {
  KeyNode, KeyboardGeometry, KeyboardMeta, KeyboardDefinition,
  MetricViolation, AnalysisReport, SearchParams, ScoringWeights,
  KeycodeDefinition, CorpusSource, BiometricSample, UserStatsStore,
  Config, SystemMetrics, JobStatus, SwapSuggestion, JobRequest
};

export type RegisterJobRequest = JobRequest;

export interface ValidationResult {
  layout_name: string;
  score: AnalysisReport;
  geometry: KeyboardGeometry;
  heatmap: number[];
  penalty_map: number[];
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



export interface StartSearchRequest {
  pinned_keys: string;
  search_params: SearchParams;
  weights: ScoringWeights;
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
