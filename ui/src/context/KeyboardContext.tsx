import { createContext, useContext, useState, useEffect, ReactNode, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
    ValidationResult, ScoringWeights, SearchParams,
    KeycodeDefinition
} from "../types";
import { keycodeService, fromDisplayString, formatForDisplay } from "../utils";

interface KeyboardContextType {
    // ... (Interface remains the same)
    weights: ScoringWeights | null;
    searchParams: SearchParams | null;
    setWeights: (w: ScoringWeights) => void;
    setSearchParams: (p: SearchParams) => void;
    keyboards: string[];
    selectedKeyboard: string;
    selectKeyboard: (name: string) => Promise<void>;
    corpora: string[];
    selectedCorpus: string;
    selectCorpus: (filename: string) => Promise<void>;
    availableLayouts: Record<string, string>;
    standardLayouts: string[];
    layoutName: string;
    layoutString: string;
    setLayoutName: (n: string) => void;
    updateLayoutString: (s: string) => void;
    loadLayoutPreset: (name: string) => void;
    activeResult: ValidationResult | null;
    referenceResult: ValidationResult | null;
    isValidating: boolean;
    refreshData: () => Promise<void>;
    saveUserLayout: (name: string) => Promise<void>;
    deleteUserLayout: (name: string) => Promise<void>;
    activeJobId: string | null;
    startJob: (id: string) => void;
    stopJob: () => void;
    selectedKeyIndex: number | null;
    setSelectedKeyIndex: (i: number | null) => void;
}

const KeyboardContext = createContext<KeyboardContextType | undefined>(undefined);

export function KeyboardProvider({ children }: { children: ReactNode }) {
    const [weights, setWeights] = useState<ScoringWeights | null>(null);
    const [searchParams, setSearchParams] = useState<SearchParams | null>(null);
    const [keyboards, setKeyboards] = useState<string[]>([]);
    const [selectedKeyboard, setSelectedKeyboard] = useState(() => localStorage.getItem("last_keyboard") || "corne");
    const [corpora, setCorpora] = useState<string[]>([]);
    const [selectedCorpus, setSelectedCorpus] = useState(() => localStorage.getItem("last_corpus") || "ngrams-all.tsv");
    const [availableLayouts, setAvailableLayouts] = useState<Record<string, string>>({});
    const [standardLayouts, setStandardLayouts] = useState<string[]>([]);
    const [layoutName, setLayoutName] = useState("Custom");
    const [layoutString, setLayoutString] = useState("");
    const [selectedKeyIndex, setSelectedKeyIndex] = useState<number | null>(null);
    const [activeResult, setActiveResult] = useState<ValidationResult | null>(null);
    const [referenceResult, setReferenceResult] = useState<ValidationResult | null>(null);
    const [isValidating, setIsValidating] = useState(false);
    const [activeJobId, setActiveJobId] = useState<string | null>(null);

    const runValidation = useCallback(async (name: string, qmkStr: string, w: ScoringWeights | null) => {
        setIsValidating(true);
        try {
            const res = await invoke<ValidationResult>("cmd_validate_layout", { layoutStr: qmkStr, weights: w });
            setActiveResult({ ...res, layoutName: name });
        } catch (e) {
            console.error("Validation error:", e);
        } finally {
            setIsValidating(false);
        }
    }, []);

    const loadKeyboard = useCallback(async (kbName: string, corpus: string) => {
        try {
            await invoke("cmd_load_dataset", { keyboardName: kbName, corpusFilename: corpus });
            const standards = await invoke<Record<string, string>>("cmd_get_loaded_layouts");
            setStandardLayouts(Object.keys(standards));
            const all = await invoke<Record<string, string>>("cmd_get_all_layouts_scoped", { keyboardId: kbName });
            setAvailableLayouts(all);

            const preferred = "Qwerty";
            const defName = all[preferred] ? preferred : Object.keys(all)[0] || "Custom";
            const qmkStr = all[defName] || "";

            setLayoutName(defName);
            setLayoutString(formatForDisplay(qmkStr));
            setSelectedKeyIndex(null);

            if (qmkStr) {
                if (all["Qwerty"]) {
                    const ref = await invoke<ValidationResult>("cmd_validate_layout", { layoutStr: all["Qwerty"], weights: null });
                    setReferenceResult(ref);
                }
                runValidation(defName, qmkStr, weights);
            }
        } catch (e) {
            console.error("Failed to load keyboard:", e);
        }
    }, [weights, runValidation]);

    useEffect(() => {
        const init = async () => {
            try {
                const conf = await invoke<{ weights: ScoringWeights, search: SearchParams }>("cmd_get_default_config");
                setWeights(conf.weights);
                setSearchParams(conf.search);

                const kbs = await invoke<string[]>("cmd_list_keyboards");
                setKeyboards(kbs);

                const corps = await invoke<string[]>("cmd_list_corpora");
                setCorpora(corps);

                const reg = await invoke<{ definitions: KeycodeDefinition[] }>("cmd_get_keycodes");
                keycodeService.loadDefinitions(reg.definitions);

                const kbToLoad = kbs.includes(selectedKeyboard) ? selectedKeyboard : (kbs[0] || "corne");
                const corpusToLoad = corps.includes(selectedCorpus) ? selectedCorpus : (corps[0] || "ngrams-all.tsv");

                if (kbToLoad !== selectedKeyboard) setSelectedKeyboard(kbToLoad);
                if (corpusToLoad !== selectedCorpus) setSelectedCorpus(corpusToLoad);

                if (kbToLoad) {
                    await loadKeyboard(kbToLoad, corpusToLoad);
                }
            } catch (e) {
                console.error("Initialization failed:", e);
            }
        };
        init();
    }, []);

    const selectKeyboard = async (name: string) => {
        setSelectedKeyboard(name);
        localStorage.setItem("last_keyboard", name);
        await loadKeyboard(name, selectedCorpus);
    };

    const selectCorpus = async (filename: string) => {
        setSelectedCorpus(filename);
        localStorage.setItem("last_corpus", filename);
        await loadKeyboard(selectedKeyboard, filename);
    };

    const updateLayoutString = (val: string) => {
        if (standardLayouts.includes(layoutName)) {
            setLayoutName("Custom");
        }
        setLayoutString(val);
    };

    const loadLayoutPreset = (name: string) => {
        setLayoutName(name);
        setSelectedKeyIndex(null);
        if (availableLayouts[name]) {
            const display = formatForDisplay(availableLayouts[name]);
            setLayoutString(display);
            runValidation(name, availableLayouts[name], weights);
        }
    };

    const saveUserLayout = async (name: string) => {
        const standardized = fromDisplayString(layoutString);
        await invoke("cmd_save_user_layout", { keyboardId: selectedKeyboard, name, layout: standardized });
        const all = await invoke<Record<string, string>>("cmd_get_all_layouts_scoped", { keyboardId: selectedKeyboard });
        setAvailableLayouts(all);
        setLayoutName(name);
    };

    const deleteUserLayout = async (name: string) => {
        await invoke("cmd_delete_user_layout", { keyboardId: selectedKeyboard, name });
        const all = await invoke<Record<string, string>>("cmd_get_all_layouts_scoped", { keyboardId: selectedKeyboard });
        setAvailableLayouts(all);
        setLayoutName("Custom");
    };

    // FIXED: Refetch the list of keyboards when refreshing data (Sync)
    const refreshData = async () => {
        const kbs = await invoke<string[]>("cmd_list_keyboards");
        setKeyboards(kbs);
        const corps = await invoke<string[]>("cmd_list_corpora");
        setCorpora(corps);

        // Reload current keyboard
        if (kbs.includes(selectedKeyboard)) {
            await loadKeyboard(selectedKeyboard, selectedCorpus);
        }
    };

    return (
        <KeyboardContext.Provider value={{
            weights, setWeights, searchParams, setSearchParams,
            keyboards, selectedKeyboard, selectKeyboard,
            corpora, selectedCorpus, selectCorpus,
            availableLayouts, standardLayouts, layoutName, layoutString,
            setLayoutName, updateLayoutString, loadLayoutPreset,
            activeResult, referenceResult, isValidating,
            refreshData, saveUserLayout, deleteUserLayout,
            activeJobId, startJob: setActiveJobId, stopJob: () => setActiveJobId(null),
            selectedKeyIndex, setSelectedKeyIndex
        }}>
            {children}
        </KeyboardContext.Provider>
    );
}

export const useKeyboard = () => {
    const ctx = useContext(KeyboardContext);
    if (!ctx) throw new Error("useKeyboard must be used within KeyboardProvider");
    return ctx;
};