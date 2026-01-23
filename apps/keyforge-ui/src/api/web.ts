import { BackendClient } from "./client";
import { AssetService } from "./services/AssetService";
import { JobService } from "./services/JobService";
import { WorkerService } from "./services/WorkerService";
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

export class WebClient implements BackendClient {
  private assets: AssetService;
  private jobs: JobService;
  private worker: WorkerService;

  constructor(hiveUrl: string = "https://api.keyforge.infodungeon.com") {
    this.assets = new AssetService(hiveUrl);
    this.jobs = new JobService(hiveUrl);
    this.worker = new WorkerService();
  }

  // --- Configuration ---
  async getDefaultConfig(hiveUrl?: string): Promise<Config> {
    return this.assets.getDefaultConfig(hiveUrl);
  }

  async getKeycodes(
    hiveUrl?: string,
  ): Promise<{ definitions: KeycodeDefinition[] }> {
    return this.assets.getKeycodes(hiveUrl);
  }

  async getUiCategories(hiveUrl?: string): Promise<any> {
    return this.assets.getUiCategories(hiveUrl);
  }

  // --- Library ---
  async listKeyboards(hiveUrl?: string): Promise<string[]> {
    return this.assets.listKeyboards(hiveUrl);
  }

  async listKeymapExtras(hiveUrl?: string): Promise<string[]> {
    return this.assets.listKeymapExtras(hiveUrl);
  }

  async getKeyboardGeometry(
    name: string,
    hiveUrl?: string,
  ): Promise<KeyboardGeometry> {
    return this.assets.getKeyboardGeometry(name, hiveUrl);
  }

  async getAllLayoutsScoped(
    keyboardId: string,
    hiveUrl?: string,
  ): Promise<Record<string, string | undefined>> {
    return this.assets.getAllLayoutsScoped(keyboardId, hiveUrl);
  }

  async saveUserLayout(
    _keyboardId: string,
    _name: string,
    _layout: string,
  ): Promise<void> {}

  async deleteUserLayout(_keyboardId: string, _name: string): Promise<void> {}

  async submitUserLayout(
    hiveUrl: string,
    hiveSecret: string,
    name: string,
    layout: string,
    author: string,
  ): Promise<string> {
    return this.jobs.submitUserLayout(
      hiveUrl,
      hiveSecret,
      name,
      layout,
      author,
    );
  }

  // --- Analysis ---
  async listCorpora(hiveUrl?: string): Promise<string[]> {
    return this.assets.listCorpora(hiveUrl);
  }

  async listCostMatrices(hiveUrl?: string): Promise<string[]> {
    return this.assets.listCostMatrices(hiveUrl);
  }

  async getCorpusStats(_hiveUrl?: string): Promise<CorpusStats[]> {
    return [];
  }

  async loadDataset(
    keyboardName: string,
    corpusFilename: string,
    costFilename: string,
    extras: string[],
    hiveUrl?: string,
  ): Promise<string> {
    const url = hiveUrl || (this.assets as any).hiveUrl;
    const corpusName = corpusFilename.replace(/\/1grams\.json$/, "");

    // We reuse fetchJson from assets or jobs (they share the same hiveUrl base)
    const [keyboardDef, keycodes, corpus, cost, config] = await Promise.all([
      (this.assets as any).fetchJson(`api/keyboards/${keyboardName}`, url),
      this.assets.getKeycodes(url),
      (this.assets as any).fetchJson(`api/corpus/${corpusName}`, url),
      (this.assets as any).fetchJson(`data/${costFilename}`, url),
      this.assets.getDefaultConfig(url),
    ]);

    this.worker.setLastKeyboardData({
      keyboardName,
      keyboardDef,
      keycodes: {
        definitions: keycodes.definitions || keycodes,
        name_to_code: {},
        code_to_label: {},
      },
      corpusName: corpusFilename,
      corpus,
      costName: costFilename,
      cost,
      weights: config.weights,
      params: config.search,
      extras,
    });

    return "Loaded";
  }

  async validateLayout(
    layoutStr: string,
    weights?: ScoringWeights,
  ): Promise<ValidationResult> {
    return this.worker.validateLayout(layoutStr, weights);
  }

  async getSmartSwaps(_layoutStr: string): Promise<SwapSuggestion[]> {
    return [];
  }

  // --- Search ---
  async dispatchJob(
    hiveUrl: string,
    hiveSecret: string,
    request: RegisterJobRequest,
  ): Promise<string> {
    return this.jobs.dispatchJob(hiveUrl, hiveSecret, request);
  }

  async pollHiveStatus(
    hiveUrl: string,
    hiveSecret: string,
    jobId: string,
  ): Promise<JobStatusUpdate> {
    return this.jobs.pollHiveStatus(hiveUrl, hiveSecret, jobId);
  }

  async stopSearch(): Promise<void> {
    this.worker.stopSearch();
  }

  async toggleLocalWorker(
    enabled: boolean,
    _url: string,
    _secret: string,
  ): Promise<string> {
    return this.worker.toggleLocalWorker(enabled);
  }

  // --- Arena ---
  async getTypingWords(
    _corpora: CorpusSource[],
    _count: number,
  ): Promise<string[]> {
    return ["the", "quick", "brown", "fox"];
  }

  async getCorpusBigrams(
    _corpora: CorpusSource[],
    _limit: number,
  ): Promise<string[]> {
    return ["th", "he", "in"];
  }

  async saveBiometrics(_samples: BiometricSample[]): Promise<string> {
    return "Saved locally";
  }

  async loadUserStats(): Promise<BiometricSample[]> {
    return [];
  }

  async resetUserStats(): Promise<string> {
    return "Reset";
  }

  async generatePersonalProfile(): Promise<string> {
    return "Generated";
  }

  // --- System ---
  async syncData(_hiveUrl: string): Promise<SyncStats> {
    return { downloaded: 0, merged: 0, skipped: 0, errors: [] };
  }

  async bootstrapAssets(hiveUrl: string): Promise<string[]> {
    const manifest = await (this.assets as any).fetchJson("manifest", hiveUrl);
    return Object.keys(manifest.files);
  }

  async getSystemHealth(): Promise<SystemHealth> {
    return {
      cpu_usage: 0,
      memory_used: 0,
      memory_total: 0,
      uptime: 0,
      cores: 1,
    };
  }

  async checkHiveHealth(hiveUrl: string): Promise<string> {
    return this.jobs.checkHiveHealth(hiveUrl);
  }

  // --- Export ---
  async exportFirmware(
    _name: string,
    _layout: string,
    _fmt: string,
  ): Promise<string> {
    return "// Export not supported in Web mode yet";
  }

  async saveFile(_path: string, _content: string): Promise<void> {}

  async parseKle(_json: string): Promise<KeyboardDefinition> {
    throw new Error("KLE Import not supported in Web Mode yet.");
  }

  async saveKeyboard(filename: string, def: KeyboardDefinition): Promise<void> {
    const blob = new Blob([JSON.stringify(def, null, 2)], {
      type: "application/json",
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `${filename}.json`;
    a.click();
    URL.revokeObjectURL(url);
  }

  async openUrl(url: string): Promise<void> {
    window.open(url, "_blank");
  }
}
