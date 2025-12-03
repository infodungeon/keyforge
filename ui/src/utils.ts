import { KeyboardGeometry, KeycodeDefinition } from "./types";

// === TYPES ===

export interface DerivedStats {
    handBalance: { left: number; right: number };
    rowUsage: { top: number; home: number; bottom: number; thumb: number };
    fingerUsage: number[];
    colUsage: { val: number; color: string }[];
}

// === SERVICE ===

class KeycodeService {
    private nameToCode: Record<string, number> = {};
    private codeToDef: Record<number, KeycodeDefinition> = {};
    private codeToLabel: Record<number, string> = {};

    constructor() {
        for (let i = 32; i <= 126; i++) {
            const char = String.fromCharCode(i);
            this.register({ code: i, id: `KC_${char}`, label: char, aliases: [char] });
        }
    }

    private register(def: KeycodeDefinition) {
        this.codeToDef[def.code] = def;
        this.codeToLabel[def.code] = def.label;
        this.nameToCode[def.id.toUpperCase()] = def.code;
        def.aliases.forEach(alias => {
            this.nameToCode[alias.toUpperCase()] = def.code;
        });
    }

    public loadDefinitions(defs: KeycodeDefinition[]) {
        this.nameToCode = {};
        this.codeToDef = {};
        this.codeToLabel = {};

        defs.forEach(d => this.register(d));

        for (let i = 32; i <= 126; i++) {
            if (!this.codeToDef[i]) {
                const char = String.fromCharCode(i);
                this.register({ code: i, id: `KC_${char}`, label: char, aliases: [char] });
            }
        }
    }

    public formatForDisplay(raw: string): string {
        if (!raw) return "";
        const tokens = raw.trim().split(/\s+/);

        return tokens.map(t => {
            const upper = t.toUpperCase();
            if (upper.startsWith("KC_") && upper.length === 4) {
                const char = upper.charAt(3);
                if (/[A-Z]/.test(char)) {
                    return char.toLowerCase();
                }
            }
            if (upper.startsWith("KC_")) {
                const short = upper.replace("KC_", "");
                if (short.length > 1) {
                    return short.charAt(0).toUpperCase() + short.slice(1).toLowerCase();
                }
                return short;
            }
            return t;
        }).join(" ");
    }

    public fromDisplayString(display: string): string {
        if (!display) return "";
        const tokens = display.trim().split(/[\s,]+/);
        const output: string[] = [];

        for (const t of tokens) {
            if (!t) continue;
            const upper = t.toUpperCase();
            if (this.nameToCode[upper] !== undefined) {
                const def = this.codeToDef[this.nameToCode[upper]];
                output.push(def ? def.id : upper);
                continue;
            }
            const withKc = `KC_${upper}`;
            if (this.nameToCode[withKc] !== undefined) {
                const def = this.codeToDef[this.nameToCode[withKc]];
                output.push(def ? def.id : withKc);
                continue;
            }
            if (t.length === 1 && /[a-zA-Z]/.test(t)) {
                output.push(`KC_${upper}`);
                continue;
            }
            output.push(upper);
        }
        return output.join(" ");
    }

    public toDisplayString(raw: string): string {
        if (!raw) return "";
        return raw.split(/\s+/).join(" ");
    }

    /**
     * Gets the short label for Keycap display.
     */
    public getVisualLabel(token: string): string {
        if (!token) return "";

        // 1. Strip common prefixes 
        let clean = token
            .replace(/^KC_/, "")
            .replace(/^Key/, "")
            .replace(/^Digit/, "")
            .replace(/^Numpad/, ""); // Covers Numpad0 -> 0

        const upper = clean.toUpperCase();

        // Visual Overrides
        const map: Record<string, string> = {
            // Modifiers
            "LCTL": "Ctrl", "RCTL": "Ctrl", "LCTRL": "Ctrl", "RCTRL": "Ctrl", "CONTROLLEFT": "Ctrl", "CONTROLRIGHT": "Ctrl",
            "LSFT": "Shift", "RSFT": "Shift", "LSHIFT": "Shift", "RSHIFT": "Shift", "SHIFTLEFT": "Shift", "SHIFTRIGHT": "Shift",
            "LALT": "Alt", "RALT": "Alt", "ALTLEFT": "Alt", "ALTRIGHT": "Alt",
            "LGUI": "Gui", "RGUI": "Gui", "LWIN": "Gui", "RWIN": "Gui", "METALEFT": "Gui", "METARIGHT": "Gui", "CMD": "Gui",
            "APP": "Menu", "CONTEXTMENU": "Menu",
            "CAPSLOCK": "Caps", "CAPS": "Caps",
            "NUMLOCK": "Num", "NLCK": "Num",
            "SCROLLLOCK": "ScrLk", "SLCK": "ScrLk",
            "PRINTSCREEN": "PrtSc", "PSCR": "PrtSc",
            "PAUSE": "Pause", "PAUS": "Pause",

            // Symbols & Actions
            "QUOT": "'", "QUOTE": "'",
            "SCLN": ";", "SEMICOLON": ";",
            "SLSH": "/", "SLASH": "/",
            "COMM": ",", "COMMA": ",",
            "DOT": ".", "PERIOD": ".",
            "LBRC": "[", "BRACKETLEFT": "[",
            "RBRC": "]", "BRACKETRIGHT": "]",
            "BSLS": "\\", "BACKSLASH": "\\",
            "MINS": "-", "MINUS": "-",
            "EQL": "=", "EQUAL": "=",
            "GRV": "`", "BACKQUOTE": "`",
            "TILD": "~",
            "EXLM": "!",
            "AT": "@", "HASH": "#", "DLR": "$", "PERC": "%", "CIRC": "^", "AMPR": "&", "ASTR": "*",
            "LPRN": "(", "RPRN": ")", "UNDS": "_", "PLUS": "+",
            "LCBR": "{", "RCBR": "}", "PIPE": "|", "COLN": ":", "DQUO": "\"",
            "QUES": "?", "LABK": "<", "RABK": ">",

            "ESC": "Esc", "ESCAPE": "Esc",
            "ENT": "Enter", "ENTER": "Enter",
            "BSPC": "Bksp", "BACKSPACE": "Bksp",
            "DEL": "Del", "DELETE": "Del",
            "INS": "Ins", "INSERT": "Ins",
            "PGUP": "PgUp", "PAGEUP": "PgUp",
            "PGDN": "PgDn", "PAGEDOWN": "PgDn",
            "SPC": "", "SPACE": "",
            "NO": "", "TRNS": "▽",

            // Arrows
            "UP": "↑", "ARROWUP": "↑",
            "DOWN": "↓", "ARROWDOWN": "↓",
            "LEFT": "←", "ARROWLEFT": "←",
            "RIGHT": "→", "ARROWRIGHT": "→", "RGHT": "→",

            // Numpad specific QMK codes
            "P0": "0", "P1": "1", "P2": "2", "P3": "3", "P4": "4",
            "P5": "5", "P6": "6", "P7": "7", "P8": "8", "P9": "9",
            "PDOT": ".", "PENT": "Enter", "PPLS": "+", "PMNS": "-", "PAST": "*", "PSLS": "/",

            // Browser Numpad codes (after strip)
            "DIVIDE": "/", "MULTIPLY": "*", "SUBTRACT": "-", "ADD": "+", "DECIMAL": "."
        };

        if (map[upper]) return map[upper];

        // Fallback: If it was just "A" or "1" originally
        return clean;
    }
}

export const keycodeService = new KeycodeService();
export function formatForDisplay(raw: string): string { return keycodeService.formatForDisplay(raw); }
export function toDisplayString(raw: string): string { return keycodeService.toDisplayString(raw); }
export function fromDisplayString(display: string): string { return keycodeService.fromDisplayString(display); }

const FINGER_COLORS = [
    "bg-slate-500", "bg-green-500", "bg-blue-500", "bg-purple-500", "bg-pink-500"
];

export function calculateStats(geo: KeyboardGeometry, heatmap: number[]): DerivedStats {
    let maxCol = 12;
    if (geo.keys.length > 0) {
        maxCol = Math.max(maxCol, ...geo.keys.map(k => k.col)) + 1;
    }

    const stats = {
        handBalance: { left: 0, right: 0 },
        rowUsage: { top: 0, home: 0, bottom: 0, thumb: 0 },
        fingerUsage: [0, 0, 0, 0, 0],
        colFingers: Array.from({ length: maxCol }, () => [0, 0, 0, 0, 0]),
        colVals: Array(maxCol).fill(0)
    };

    let total = 0;
    geo.keys.forEach((k, i) => {
        const val = heatmap[i] || 0;
        if (val === 0) return;
        total += val;
        if (k.hand === 0) stats.handBalance.left += val; else stats.handBalance.right += val;
        if (k.finger >= 0 && k.finger <= 4) stats.fingerUsage[k.finger] += val;
        if (k.row === geo.home_row && k.finger !== 0) stats.rowUsage.home += val;
        else if (k.row < geo.home_row && k.finger !== 0) stats.rowUsage.top += val;
        else if (k.row > geo.home_row && k.finger !== 0) stats.rowUsage.bottom += val;
        else if (k.finger === 0) stats.rowUsage.thumb += val;
        if (k.col >= 0 && k.col < maxCol) {
            stats.colVals[k.col] += val;
            if (k.finger >= 0 && k.finger <= 4) {
                stats.colFingers[k.col][k.finger]++;
            }
        }
    });

    const norm = (v: number) => total > 0 ? (v / total) * 100 : 0;
    const colUsage = stats.colVals.map((val, idx) => {
        const fingers = stats.colFingers[idx];
        const dominantFinger = fingers.indexOf(Math.max(...fingers));
        return {
            val: norm(val),
            color: FINGER_COLORS[dominantFinger] || "bg-slate-700"
        };
    });
    return {
        handBalance: { left: norm(stats.handBalance.left), right: norm(stats.handBalance.right) },
        rowUsage: { top: norm(stats.rowUsage.top), home: norm(stats.rowUsage.home), bottom: norm(stats.rowUsage.bottom), thumb: norm(stats.rowUsage.thumb) },
        fingerUsage: stats.fingerUsage.map(norm),
        colUsage
    };
}