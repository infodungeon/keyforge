import { invoke } from "@tauri-apps/api/core";
import { BackendError } from "./error";
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

export class TauriClient implements BackendClient {
  private cache = new Map<string, Promise<any>>();

  private memoize<T>(key: string, fn: () => Promise<T>): Promise<T> {
    if (this.cache.has(key)) {
      return this.cache.get(key) as Promise<T>;
    }
    const promise = fn().catch((e) => {
      this.cache.delete(key);
      const err = BackendError.from(e);
      console.error(`[API] ${key} failed:`, err);
      throw err;
    });
    this.cache.set(key, promise);
    return promise;
  }

  async getDefaultConfig(_hiveUrl?: string): Promise<Config> {
    return this.memoize("config", () => this.call("cmd_get_default_config"));
  }

  async getKeycodes(
    _hiveUrl?: string,
  ): Promise<{ definitions: KeycodeDefinition[] }> {
    return this.memoize("keycodes", async () => {
      const data = await this.call<any>("cmd_get_keycodes");
      return Array.isArray(data) ? { definitions: data } : data;
    });
  }

  async getUiCategories(_hiveUrl?: string): Promise<any> {
    return this.memoize("ui_categories", () => this.call("cmd_get_ui_categories"));
  }

  async listKeyboards(_hiveUrl?: string): Promise<string[]> {
    return this.call("cmd_list_keyboards");
  }

  async listKeymapExtras(_hiveUrl?: string): Promise<string[]> {
    return this.call("cmd_list_keymap_extras");
  }

  async getKeyboardGeometry(
    name: string,
    _hiveUrl?: string,
  ): Promise<KeyboardGeometry> {
    return this.memoize(`geo:${name}`, () =>
      this.call("cmd_get_keyboard_geometry", { name }),
    );
  }

  async getAllLayoutsScoped(
    keyboardId: string,
    _hiveUrl?: string,
  ): Promise<Record<string, string>> {
    return this.call("cmd_get_all_layouts_scoped", { keyboardId });
  }

  async saveUserLayout(
    keyboardId: string,
    name: string,
    layout: string,
    _hiveUrl?: string,
  ): Promise<void> {
    return this.call("cmd_save_user_layout", { keyboardId, name, layout });
  }

  async deleteUserLayout(
    keyboardId: string,
    name: string,
    _hiveUrl?: string,
  ): Promise<void> {
    return this.call("cmd_delete_user_layout", { keyboardId, name });
  }

  async submitUserLayout(
    hiveUrl: string,
    hiveSecret: string,
    name: string,
    layout: string,
    author: string,
  ): Promise<string> {
    return this.call("cmd_submit_user_layout", {
      hiveUrl,
      hiveSecret,
      name,
      layout,
      author,
    });
  }

  async listCorpora(_hiveUrl?: string): Promise<string[]> {
    return this.call("cmd_list_corpora");
  }

  async listCostMatrices(_hiveUrl?: string): Promise<string[]> {
    return this.call("cmd_list_cost_matrices");
  }

  async getCorpusStats(_hiveUrl?: string): Promise<CorpusStats[]> {
    return this.call("cmd_get_corpus_stats");
  }

  async loadDataset(
    keyboardName: string,
    corpusFilename: string,
    costFilename: string,
    extras: string[],
    _hiveUrl?: string,
  ): Promise<string> {
    return this.call("cmd_load_dataset", {
      keyboardName,
      corpusFilename,
      costFilename,
      extras,
    });
  }

  async validateLayout(
    layoutStr: string,
    weights?: ScoringWeights,
    _hiveUrl?: string,
    keyboardName?: string,
  ): Promise<ValidationResult> {
    return this.call("cmd_validate_layout", { layoutStr, weights, keyboardName });
  }

  async getSmartSwaps(
    layoutStr: string,
    _hiveUrl?: string,
  ): Promise<SwapSuggestion[]> {
    return this.call("cmd_get_smart_swaps", { layoutStr });
  }

  async dispatchJob(
    hiveUrl: string,
    hiveSecret: string,
    request: RegisterJobRequest,
  ): Promise<string> {
    return this.call("cmd_dispatch_job", { hiveUrl, hiveSecret, request });
  }

  async pollHiveStatus(
    hiveUrl: string,
    hiveSecret: string,
    jobId: string,
  ): Promise<JobStatusUpdate> {
    return this.call("cmd_poll_hive_status", { hiveUrl, hiveSecret, jobId });
  }

  async stopSearch(): Promise<void> {
    return this.call("cmd_stop_search");
  }

  async toggleLocalWorker(
    enabled: boolean,
    hiveUrl: string,
    hiveSecret: string,
  ): Promise<string> {
    return this.call("cmd_toggle_local_worker", { enabled, hiveUrl, hiveSecret });
  }

  async getTypingWords(
    corpora: CorpusSource[],
    count: number,
  ): Promise<string[]> {
    return this.call("cmd_get_typing_words", { corpora, count });
  }

  async getCorpusBigrams(
    corpora: CorpusSource[],
    limit: number,
  ): Promise<string[]> {
    return this.call("cmd_get_corpus_bigrams", { corpora, limit });
  }

  async saveBiometrics(samples: BiometricSample[]): Promise<string> {
    return this.call("cmd_save_biometrics", { samples });
  }

  async loadUserStats(): Promise<BiometricSample[]> {
    return this.call("cmd_load_user_stats");
  }

  async resetUserStats(): Promise<string> {
    return this.call("cmd_reset_user_stats");
  }

  async generatePersonalProfile(): Promise<string> {
    return this.call("cmd_generate_personal_profile");
  }

  async syncData(hiveUrl: string): Promise<SyncStats> {
    return this.call("cmd_sync_data", { hiveUrl });
  }

  async bootstrapAssets(hiveUrl: string): Promise<string[]> {
    return this.call("cmd_bootstrap_assets", { hiveUrl });
  }

  async getSystemHealth(): Promise<SystemHealth> {
    return this.call("cmd_get_system_health");
  }

  async checkHiveHealth(hiveUrl: string): Promise<string> {
    return this.call("cmd_check_hive_health", { hiveUrl });
  }

  async exportFirmware(
    layoutName: string,
    layoutStr: string,
    format: string,
  ): Promise<string> {
    return this.call("cmd_export_firmware", { layoutName, layoutStr, format });
  }

  async saveFile(path: string, content: string): Promise<void> {
    return this.call("cmd_safe_write_file", { path, content });
  }

  async parseKle(json: string): Promise<KeyboardDefinition> {
    return this.call("cmd_parse_kle", { json });
  }

  async saveKeyboard(filename: string, def: KeyboardDefinition): Promise<void> {
    return this.call("cmd_save_keyboard", { filename, def });
  }

  async openUrl(url: string): Promise<void> {
    return this.call("plugin:opener|open", { path: url });
  }

  private async call<T>(cmd: string, args?: any): Promise<T> {
    try {
      return await invoke<T>(cmd, args);
    } catch (e) {
      const err = BackendError.from(e);
      console.error(`[API] ${cmd} failed:`, err);
      throw err;
    }
  }
}
