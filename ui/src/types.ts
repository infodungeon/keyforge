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
  geoDist: number;
  fingerUse: number;

  // Mechanics (Weighted)
  mechSfr: number;
  mechSfb: number;
  mechScis: number;

  // Flow (Weighted)
  flowCost: number;

  // Statistics (Raw Counts)
  totalChars: number;
  totalBigrams: number;
  totalTrigrams: number;

  statSfb: number;
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
  statRollIn: number;
  statRollOut: number;
  statRollTri: number;
  statRoll3In: number;
  statRoll3Out: number;

  statRedir: number;
}

export interface ValidationResult {
  layoutName: string;
  score: ScoreDetails;
  geometry: KeyboardGeometry;
  heatmap: number[];
}