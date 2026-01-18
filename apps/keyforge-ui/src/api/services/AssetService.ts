// apps/keyforge-ui/src/api/services/AssetService.ts

import { BaseService } from "./BaseService";
import {
    Config,
    KeycodeDefinition,
    KeyboardGeometry,
    KeyboardDefinition,
} from "../../types";
import { DEFAULT_APP_CONFIG } from "../constants";

export class AssetService extends BaseService {
    async getDefaultConfig(hiveUrl?: string): Promise<Config> {
        return this.fetchJson<Config>("data/config.json", hiveUrl).catch(
            () => DEFAULT_APP_CONFIG,
        );
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

    async listCorpora(hiveUrl?: string): Promise<string[]> {
        return this.fetchJson<string[]>("api/corpora", hiveUrl);
    }

    async listCostMatrices(hiveUrl?: string): Promise<string[]> {
        return this.fetchJson<string[]>("api/costs", hiveUrl);
    }
}
