import { KeycodeDefinition } from "../types";

class KeycodeService {
  private nameToCode: Record<string, number> = {};
  private codeToDef: Record<number, KeycodeDefinition> = {};
  private codeToLabel: Record<number, string> = {};
  private codeToShortName: Record<number, string> = {};
  private isLoaded = false;

  constructor() {
    // No hardcoded data. Must call loadDefinitions.
  }

  private register(def: KeycodeDefinition) {
    this.codeToDef[def.code] = def;
    this.codeToLabel[def.code] = def.label;

    const candidates = [def.id, ...def.aliases];

    candidates.forEach((name) => {
      this.nameToCode[name.toUpperCase()] = def.code;
    });

    // Determine Shortest Alias for Internal Use
    candidates.sort((a, b) => a.length - b.length || a.localeCompare(b));
    this.codeToShortName[def.code] = candidates[0];
  }

  public loadDefinitions(defs: KeycodeDefinition[]) {
    if (!defs || defs.length === 0) {
      console.error("[KeycodeService] Received empty definitions!");
      throw new Error(
        "Keycode Registry Initialization Failed: No definitions provided.",
      );
    }
    this.nameToCode = {};
    this.codeToDef = {};
    this.codeToLabel = {};
    this.codeToShortName = {};

    defs.forEach((d) => this.register(d));
    this.isLoaded = true;
    console.log(`[KeycodeService] Loaded ${defs.length} definitions.`);
  }

  // Visual Label (Keycap) - Strictly from Registry
  public getVisualLabel(token: string): string {
    if (!this.isLoaded) {
      console.warn("[KeycodeService] Accessing before load:", token);
      return token || "???";
    }
    if (!token) return "";

    const upper = token.toUpperCase();
    const code = this.nameToCode[upper];

    if (code !== undefined) {
      const label = this.codeToLabel[code];
      if (label) return label;
    }

    // Fallback: Return raw token but warn
    // console.warn(`[KeycodeService] Unknown key: ${token}`);
    return token;
  }

  // Internal Data (Shortest Alias)
  public formatForDisplay(raw: string): string {
    if (!raw) return "";
    const tokens = raw.trim().split(/\s+/);

    return tokens
      .map((t) => {
        const code = this.nameToCode[t.toUpperCase()];
        if (code !== undefined && this.codeToShortName[code]) {
          return this.codeToShortName[code];
        }
        return t;
      })
      .join(" ");
  }

  public fromDisplayString(display: string): string {
    if (!display) return "";
    const tokens = display.trim().split(/\s+/);

    return tokens
      .map((t) => {
        const code = this.nameToCode[t.toUpperCase()];
        if (code !== undefined && this.codeToDef[code]) {
          return this.codeToDef[code].id;
        }
        return t;
      })
      .join(" ");
  }

  public toDisplayString(raw: string): string {
    return this.formatForDisplay(raw);
  }
}

export const keycodeService = new KeycodeService();
