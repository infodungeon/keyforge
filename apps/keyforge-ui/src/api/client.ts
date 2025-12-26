import {
  Config,
  KeycodeDefinition,
  KeyboardGeometry,
  CorpusStats,
  ValidationResult,
  SwapSuggestion,
  JobStatusUpdate,
  RegisterJobRequest,
  BiometricSample,
  SyncStats,
  SystemHealth,
  CorpusSource,
  ScoringWeights,
  KeyboardDefinition,
} from "../types";

export interface BackendClient {
  // --- Configuration ---
  getDefaultConfig(hiveUrl?: string): Promise<Config>;
  getKeycodes(hiveUrl?: string): Promise<{ definitions: KeycodeDefinition[] }>;
  getUiCategories(hiveUrl?: string): Promise<any>;

  // --- Library ---
  listKeyboards(hiveUrl?: string): Promise<string[]>;
  listKeymapExtras(hiveUrl?: string): Promise<string[]>;
  getKeyboardGeometry(
    name: string,
    hiveUrl?: string,
  ): Promise<KeyboardGeometry>;
  getAllLayoutsScoped(
    keyboardId: string,
    hiveUrl?: string,
  ): Promise<Record<string, string>>;
  saveUserLayout(
    keyboardId: string,
    name: string,
    layout: string,
    hiveUrl?: string,
  ): Promise<void>;
  deleteUserLayout(
    keyboardId: string,
    name: string,
    hiveUrl?: string,
  ): Promise<void>;
  submitUserLayout(
    hiveUrl: string,
    hiveSecret: string,
    name: string,
    layout: string,
    author: string,
  ): Promise<string>;

  // --- Analysis ---
  listCorpora(hiveUrl?: string): Promise<string[]>;
  listCostMatrices(hiveUrl?: string): Promise<string[]>;
  getCorpusStats(hiveUrl?: string): Promise<CorpusStats[]>;

  loadDataset(
    keyboardName: string,
    corpusFilename: string,
    costFilename: string,
    extras: string[],
    hiveUrl?: string,
  ): Promise<string>;
  validateLayout(
    layoutStr: string,
    weights?: ScoringWeights,
    hiveUrl?: string,
    keyboardName?: string,
  ): Promise<ValidationResult>;
  getSmartSwaps(layoutStr: string, hiveUrl?: string): Promise<SwapSuggestion[]>;

  // --- Search ---
  dispatchJob(
    hiveUrl: string,
    hiveSecret: string,
    request: RegisterJobRequest,
  ): Promise<string>;
  pollHiveStatus(
    hiveUrl: string,
    hiveSecret: string,
    jobId: string,
  ): Promise<JobStatusUpdate>;
  stopSearch(): Promise<void>;
  toggleLocalWorker(
    enabled: boolean,
    hiveUrl: string,
    hiveSecret: string,
  ): Promise<string>;

  // --- Arena ---
  getTypingWords(corpora: CorpusSource[], count: number): Promise<string[]>;
  getCorpusBigrams(corpora: CorpusSource[], limit: number): Promise<string[]>;
  saveBiometrics(samples: BiometricSample[]): Promise<string>;
  loadUserStats(): Promise<BiometricSample[]>;
  resetUserStats(): Promise<string>;
  generatePersonalProfile(): Promise<string>;

  // --- System ---
  syncData(hiveUrl: string): Promise<SyncStats>;
  bootstrapAssets(hiveUrl: string): Promise<string[]>;
  getSystemHealth(): Promise<SystemHealth>;
  checkHiveHealth(hiveUrl: string): Promise<string>;

  // --- Export ---
  exportFirmware(
    layoutName: string,
    layoutStr: string,
    format: string,
  ): Promise<string>;
  saveFile(path: string, content: string): Promise<void>;
  parseKle(json: string): Promise<KeyboardDefinition>;
  saveKeyboard(filename: string, def: KeyboardDefinition): Promise<void>;
  openUrl(url: string): Promise<void>;
}
