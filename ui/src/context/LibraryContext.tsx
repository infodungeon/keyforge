import { createContext, useContext, useState, useEffect, ReactNode, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ScoringWeights, SearchParams, KeycodeDefinition } from "../types";
import { keycodeService } from "../utils";
import { useToast } from "./ToastContext";

interface LibraryContextType {
    weights: ScoringWeights | null;
    searchParams: SearchParams | null;
    setWeights: (w: ScoringWeights) => void;
    setSearchParams: (p: SearchParams) => void;

    keyboards: string[];
    selectedKeyboard: string;
    selectKeyboard: (name: string) => void;

    corpora: string[];
    selectedCorpus: string;
    selectCorpus: (filename: string) => void;

    availableLayouts: Record<string, string>;
    standardLayouts: string[];

    refreshLibrary: () => Promise<void>;

    saveUserLayout: (name: string, layout: string) => Promise<void>;
    deleteUserLayout: (name: string) => Promise<void>;

    libraryVersion: number;
}

const LibraryContext = createContext<LibraryContextType | undefined>(undefined);

export function LibraryProvider({ children }: { children: ReactNode }) {
    const { addToast } = useToast();

    const [weights, setWeights] = useState<ScoringWeights | null>(null);
    const [searchParams, setSearchParams] = useState<SearchParams | null>(null);

    const [keyboards, setKeyboards] = useState<string[]>([]);
    const [selectedKeyboard, setSelectedKeyboard] = useState(() => localStorage.getItem("last_keyboard") || "ortho_30");

    const [corpora, setCorpora] = useState<string[]>([]);
    const [selectedCorpus, setSelectedCorpus] = useState(() => localStorage.getItem("last_corpus") || "ngrams-all.tsv");

    const [availableLayouts, setAvailableLayouts] = useState<Record<string, string>>({});
    const [standardLayouts, setStandardLayouts] = useState<string[]>([]);

    const [libraryVersion, setLibraryVersion] = useState(0);

    const refreshLibrary = useCallback(async () => {
        try {
            const [kbs, corps] = await Promise.all([
                invoke<string[]>("cmd_list_keyboards"),
                invoke<string[]>("cmd_list_corpora")
            ]);

            setKeyboards(kbs);
            setCorpora(corps);

            // Auto-select valid defaults if current selection is invalid
            if (kbs.length > 0 && !kbs.includes(selectedKeyboard)) {
                console.warn(`Selected keyboard '${selectedKeyboard}' not found. Defaulting to '${kbs[0]}'`);
                setSelectedKeyboard(kbs[0]);
            }
            if (corps.length > 0 && !corps.includes(selectedCorpus)) {
                console.warn(`Selected corpus '${selectedCorpus}' not found. Defaulting to '${corps[0]}'`);
                setSelectedCorpus(corps[0]);
            }

            setLibraryVersion(v => v + 1);
        } catch (e) {
            console.error("Library Refresh Error:", e);
            addToast('error', "Failed to load library data. Check logs.");
        }
    }, [selectedKeyboard, selectedCorpus, addToast]);

    // Initial Load
    useEffect(() => {
        const init = async () => {
            try {
                const conf = await invoke<{ weights: ScoringWeights, search: SearchParams }>("cmd_get_default_config");
                setWeights(conf.weights);
                setSearchParams(conf.search);

                const reg = await invoke<{ definitions: KeycodeDefinition[] }>("cmd_get_keycodes");
                keycodeService.loadDefinitions(reg.definitions);

                await refreshLibrary();
            } catch (e) {
                console.error("Library Init Error:", e);
                addToast('error', "Failed to initialize backend configuration.");
            }
        };
        init();
    }, [refreshLibrary, addToast]);

    // Load Keyboard Data when selection changes
    useEffect(() => {
        if (!selectedKeyboard) return;
        const loadKb = async () => {
            try {
                const all = await invoke<Record<string, string>>("cmd_get_all_layouts_scoped", { keyboardId: selectedKeyboard });

                setAvailableLayouts(all);

                // Identify standard layouts
                setStandardLayouts(Object.keys(all).filter(k => k !== "Custom"));

            } catch (e) {
                console.error("Keyboard Load Error:", e);
            }
        };
        loadKb();
    }, [selectedKeyboard, libraryVersion]);

    const selectKeyboard = (name: string) => {
        setSelectedKeyboard(name);
        localStorage.setItem("last_keyboard", name);
        // Force refresh to ensure layouts update
        setLibraryVersion(v => v + 1);
    };

    const selectCorpus = (filename: string) => {
        setSelectedCorpus(filename);
        localStorage.setItem("last_corpus", filename);
        setLibraryVersion(v => v + 1);
    };

    const saveUserLayout = async (name: string, layout: string) => {
        try {
            await invoke("cmd_save_user_layout", { keyboardId: selectedKeyboard, name, layout });
            addToast('success', `Layout '${name}' saved.`);
            // Refresh local cache
            const all = await invoke<Record<string, string>>("cmd_get_all_layouts_scoped", { keyboardId: selectedKeyboard });
            setAvailableLayouts(all);
        } catch (e) {
            addToast('error', `Save failed: ${e}`);
        }
    };

    const deleteUserLayout = async (name: string) => {
        try {
            await invoke("cmd_delete_user_layout", { keyboardId: selectedKeyboard, name });
            addToast('info', `Layout '${name}' deleted.`);
            const all = await invoke<Record<string, string>>("cmd_get_all_layouts_scoped", { keyboardId: selectedKeyboard });
            setAvailableLayouts(all);
        } catch (e) {
            addToast('error', `Delete failed: ${e}`);
        }
    };

    return (
        <LibraryContext.Provider value={{
            weights, searchParams, setWeights, setSearchParams,
            keyboards, selectedKeyboard, selectKeyboard,
            corpora, selectedCorpus, selectCorpus,
            availableLayouts, standardLayouts,
            refreshLibrary, saveUserLayout, deleteUserLayout,
            libraryVersion
        }}>
            {children}
        </LibraryContext.Provider>
    );
}

export const useLibrary = () => {
    const ctx = useContext(LibraryContext);
    if (!ctx) throw new Error("useLibrary must be used within LibraryProvider");
    return ctx;
};