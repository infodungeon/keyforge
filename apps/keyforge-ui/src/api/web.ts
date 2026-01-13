import { BackendClient } from "./client";
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
  private hiveUrl: string;
  private cache: Map<string, Promise<any>> = new Map();
  private worker: Worker | null = null;
  private workerReady = false;
  private lastKeyboardData: any = null;

  constructor(hiveUrl: string = "https://api.keyforge.infodungeon.com") {
    this.hiveUrl = hiveUrl;
    this.initWorker();
  }

  private initWorker() {
    if (typeof Worker !== "undefined") {
      this.worker = new Worker(new URL("./worker.ts", import.meta.url), {
        type: "module",
      });
      this.worker.onmessage = (e) => {
        const { type, payload } = e.data;
        if (type === "READY") {
          this.workerReady = true;
          if (this.lastKeyboardData) {
            this.worker?.postMessage({
              type: "LOAD_DATA",
              payload: this.lastKeyboardData,
            });
          }
        }
      };
      this.worker.postMessage({ type: "INIT" });
    }
  }

  private async fetchJson<T>(
    endpoint: string,
    hiveUrl?: string,
    options?: RequestInit,
  ): Promise<T> {
    const url = hiveUrl || this.hiveUrl;
    const fullUrl = `${url}/${endpoint}`;
    const isGet = !options?.method || options.method === "GET";

    if (isGet) {
      if (this.cache.has(fullUrl)) {
        return this.cache.get(fullUrl) as Promise<T>;
      }
    }

    const promise = (async () => {
      try {
        const res = await fetch(fullUrl, options);
        if (!res.ok) {
          const text = await res.text();
          console.error(`[BACKEND ERROR] ${res.status} ${fullUrl}:`, text);
          throw new Error(`Backend Error (${res.status}): ${text}`);
        }
        return await res.json();
      } catch (e) {
        if (isGet) this.cache.delete(fullUrl);
        console.error(`[FETCH FAILURE] ${fullUrl}:`, e);
        throw e;
      }
    })();

    if (isGet) {
      this.cache.set(fullUrl, promise);
    }

    return promise;
  }

  async getDefaultConfig(hiveUrl?: string): Promise<Config> {
    return this.fetchJson<Config>("data/config.json", hiveUrl).catch(() => ({
      search: {
        search_epochs: 10000,
        search_steps: 100000,
        search_patience: 500,
        search_patience_threshold: 0.1,
        temp_min: 0.005,
        temp_max: 20.0,
        opt_limit_fast: 100,
        opt_limit_slow: 1500,
        reheats: 3,
        reheat_factor: 0.5,
      },
      weights: {
        penalty_sfr_weak_finger: 20.0,
        penalty_sfr_bad_row: 25.0,
        penalty_sfr_lat: 40.0,
        penalty_sfb_lateral: 65.0,
        penalty_sfb_lateral_weak: 160.0,
        penalty_sfb_base: 400.0,
        penalty_sfb_outward_adder: 10.0,
        penalty_sfb_diagonal: 240.0,
        penalty_sfb_long: 280.0,
        penalty_sfb_bottom: 45.0,
        weight_weak_finger_sfb: 2.7,
        threshold_sfb_long_row_diff: 2,
        threshold_scissor_row_diff: 2,
        threshold_reach_stretch: 1.2,
        penalty_scissor: 25.0,
        penalty_ring_pinky: 1.3,
        penalty_lateral: 50.0,
        penalty_monogram_stretch: 20.0,
        penalty_skip: 20.0,
        penalty_redirect: 65.0,
        penalty_hand_run: 5.0,
        bonus_inward_roll: 40.0,
        bonus_bigram_roll_in: 35.0,
        bonus_bigram_roll_out: 25.0,
        penalty_high_in_med: 12.0,
        penalty_high_in_low: 20.0,
        penalty_med_in_prime: 2.0,
        penalty_med_in_low: 2.0,
        penalty_low_in_prime: 15.0,
        penalty_low_in_med: 2.0,
        penalty_imbalance: 200.0,
        max_hand_imbalance: 0.55,
        weight_vertical_travel: 1.0,
        weight_lateral_travel: 3.5,
        weight_finger_effort: 2.2,
        default_cost_ms: 120.0,
        loader_trigram_limit: 3000,
        trigram_coverage: 0.99,
        finger_penalty_scale: "1.0,1.0,1.1,1.3,1.6",
        comfortable_scissors: "21,23,34",
      },
      defs: {
        tier_high_chars: "etaoinshr",
        tier_med_chars: "ldcumwfgypb.,",
        tier_low_chars: "vkjxqz/;",
        critical_bigrams: "th,he,in,er,an,re,nd,ou",
        finger_repeat_scale: "1.0,1.0,1.0,1.2,1.5",
      },
    }));
  }

  async getKeycodes(
    hiveUrl?: string,
  ): Promise<{ definitions: KeycodeDefinition[] }> {
    try {
      const data = await this.fetchJson<any>("data/keycodes.json", hiveUrl);
      return Array.isArray(data) ? { definitions: data } : data;
    } catch (e) {
      return { definitions: [] };
    }
  }

  async getUiCategories(hiveUrl?: string): Promise<any> {
    try {
      return await this.fetchJson<any>("data/ui_categories.json", hiveUrl);
    } catch (e) {
      return {};
    }
  }

  async listKeyboards(hiveUrl?: string): Promise<string[]> {
    return this.fetchJson<string[]>("api/keyboards", hiveUrl);
  }

  async listKeymapExtras(hiveUrl?: string): Promise<string[]> {
    return this.fetchJson<string[]>("api/keymap_extras", hiveUrl);
  }

  async getKeyboardGeometry(
    name: string,
    hiveUrl?: string,
  ): Promise<KeyboardGeometry> {
    const def = await this.fetchJson<KeyboardDefinition>(
      `api/keyboards/${name}`,
      hiveUrl,
    );
    return def.geometry;
  }

  async getAllLayoutsScoped(
    keyboardId: string,
    hiveUrl?: string,
  ): Promise<Record<string, string | undefined>> {
    try {
      const def = await this.fetchJson<KeyboardDefinition>(
        `api/keyboards/${keyboardId}`,
        hiveUrl,
      );
      return def.layouts || {};
    } catch (e) {
      return {};
    }
  }

  async saveUserLayout(
    _keyboardId: string,
    _name: string,
    _layout: string,
  ): Promise<void> { }

  async deleteUserLayout(_keyboardId: string, _name: string): Promise<void> { }

  async submitUserLayout(
    hiveUrl: string,
    _hiveSecret: string,
    name: string,
    layout: string,
    author: string,
  ): Promise<string> {
    const res = await fetch(`${hiveUrl}/submissions`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ name, layout, author }),
    });
    if (!res.ok) throw new Error("Submission failed");
    return "Submitted";
  }

  async listCorpora(hiveUrl?: string): Promise<string[]> {
    return this.fetchJson<string[]>("api/corpora", hiveUrl);
  }

  async listCostMatrices(hiveUrl?: string): Promise<string[]> {
    return this.fetchJson<string[]>("api/costs", hiveUrl);
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
    const url = hiveUrl || this.hiveUrl;
    const corpusName = corpusFilename.replace(/\/1grams\.json$/, "");

    const [keyboardDef, keycodes, corpus, cost, config] = await Promise.all([
      this.fetchJson<KeyboardDefinition>(`api/keyboards/${keyboardName}`, url),
      this.getKeycodes(url),
      this.fetchJson<any>(`api/corpus/${corpusName}`, url),
      this.fetchJson<any>(`data/${costFilename}`, url),
      this.getDefaultConfig(url),
    ]);

    this.lastKeyboardData = {
      keyboardName,
      keyboardDef: keyboardDef,
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
    };

    if (this.worker && this.workerReady) {
      this.worker.postMessage({
        type: "LOAD_DATA",
        payload: this.lastKeyboardData,
      });
    }

    return "Loaded";
  }

  async validateLayout(
    layoutStr: string,
    _weights?: ScoringWeights,
    _hiveUrl?: string,
    _keyboardName?: string,
  ): Promise<ValidationResult> {
    if (!this.worker) throw new Error("Worker not initialized");
    
    return new Promise((resolve, reject) => {
      const handler = (e: MessageEvent) => {
        const { type, payload } = e.data;
        if (type === "VALIDATION_RESULT") {
          this.worker?.removeEventListener("message", handler);
          resolve(payload);
        } else if (type === "ERROR") {
          this.worker?.removeEventListener("message", handler);
          reject(new Error(payload));
        }
      };
      this.worker.addEventListener("message", handler);
      this.worker.postMessage({ type: "VALIDATE", payload: { layoutStr } });
    });
  }

  async getSmartSwaps(_layoutStr: string): Promise<SwapSuggestion[]> {
    return [];
  }

  async dispatchJob(
    hiveUrl: string,
    hiveSecret: string,
    request: RegisterJobRequest,
  ): Promise<string> {
    const res = await fetch(`${hiveUrl}/jobs`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "X-Keyforge-Secret": hiveSecret,
      },
      body: JSON.stringify(request),
    });
    const data = await res.json();
    return data.job_id;
  }

  async pollHiveStatus(
    hiveUrl: string,
    hiveSecret: string,
    jobId: string,
  ): Promise<JobStatusUpdate> {
    const res = await fetch(`${hiveUrl}/jobs/${jobId}/status`, {
      headers: { "X-Keyforge-Secret": hiveSecret },
    });
    return res.json();
  }

  async stopSearch(): Promise<void> {
    if (this.worker) {
      this.worker.postMessage({ type: "STOP" });
    }
  }

  async toggleLocalWorker(
    enabled: boolean,
    _url: string,
    _secret: string,
  ): Promise<string> {
    if (enabled) {
      if (!this.worker) this.initWorker();
      return "Web Worker Started";
    } else {
      if (this.worker) {
        this.worker.terminate();
        this.worker = null;
        this.workerReady = false;
      }
      return "Web Worker Stopped";
    }
  }

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

  async syncData(_hiveUrl: string): Promise<SyncStats> {
    return { downloaded: 0, merged: 0, skipped: 0, errors: [] };
  }

  async bootstrapAssets(hiveUrl: string): Promise<string[]> {
    const manifest = await this.fetchJson<{ files: Record<string, string> }>(
      "manifest",
      hiveUrl,
    );
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
    const res = await fetch(`${hiveUrl}/health`);
    if (!res.ok) throw new Error("Hive unhealthy");
    return "OK";
  }

  async exportFirmware(
    _name: string,
    _layout: string,
    _fmt: string,
  ): Promise<string> {
    return "// Export not supported in Web mode yet";
  }

  async saveFile(_path: string, _content: string): Promise<void> { }

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
