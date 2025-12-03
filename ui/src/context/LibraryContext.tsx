import { createContext, useContext, useState, useEffect, ReactNode, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ScoringWeights, SearchParams, KeycodeDefinition } from "../types";
import { keycodeService } from "../utils";

interface LibraryContextType {
    weights: ScoringWeights | null;
    searchParams: SearchParams | null;
    setWeights: (w: ScoringWeights) => void;
    setSearchParams: (p: SearchParams) => void;
    
    keyboards: string[];
    selectedKeyboard: string;
    selectKeyboard: (name: string) => void; // Updated to not promise void for simpler consumption
    
    corpora: string[];
    selectedCorpus: string;
    selectCorpus: (filename: string) => void;
    
    availableLayouts: Record<string, string>;
    standardLayouts: string[];
    
    refreshLibrary: () => Promise<void>;
    
    // Actions that mutate library state
    saveUserLayout: (name: string, layout: string) => Promise<void>;
    deleteUserLayout: (name: string) => Promise<void>;
    
    // Internal trigger for Session to reload
    libraryVersion: number;
}

const LibraryContext = createContext<LibraryContextType | undefined>(undefined);

export function LibraryProvider({ children }: { children: ReactNode }) {
    const [weights, setWeights] = useState<ScoringWeights | null>(null);
    const [searchParams, setSearchParams] = useState<SearchParams | null>(null);
    const [keyboards, setKeyboards] = useState<string[]>([]);
    const [selectedKeyboard, setSelectedKeyboard] = useState(() => localStorage.getItem("last_keyboard") || "corne");
    const [corpora, setCorpora] = useState<string[]>([]);
    const [selectedCorpus, setSelectedCorpus] = useState(() => localStorage.getItem("last_corpus") || "ngrams-all.tsv");
    
    const [availableLayouts, setAvailableLayouts] = useState<Record<string, string>>({});
    const [standardLayouts, setStandardLayouts] = useState<string[]>([]);
    
    // Simple counter to notify dependents that data has changed (like a reload signal)
    const [libraryVersion, setLibraryVersion] = useState(0);

    const refreshLibrary = useCallback(async () => {
        try {
            const kbs = await invoke<string[]>("cmd_list_keyboards");
            setKeyboards(kbs);
            const corps = await invoke<string[]>("cmd_list_corpora");
            setCorpora(corps);
            
            // Ensure valid selections
            if (!kbs.includes(selectedKeyboard) && kbs.length > 0) setSelectedKeyboard(kbs[0]);
            if (!corps.includes(selectedCorpus) && corps.length > 0) setSelectedCorpus(corps[0]);
            
            setLibraryVersion(v => v + 1);
        } catch (e) {
            console.error("Library Refresh Error:", e);
        }
    }, [selectedKeyboard, selectedCorpus]);

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
            }
        };
        init();
    }, [refreshLibrary]);

    // Load Keyboard Data when selection changes
    useEffect(() => {
        if (!selectedKeyboard) return;
        const loadKb = async () => {
            try {
                // Pre-load logic (backend state update)
                // Note: The session actually does the heavy lifting of validation, 
                // but Library fetches the definitions.
                const standards = await invoke<Record<string, string>>("cmd_get_loaded_layouts");
                setStandardLayouts(Object.keys(standards));
                
                const all = await invoke<Record<string, string>>("cmd_get_all_layouts_scoped", { keyboardId: selectedKeyboard });
                setAvailableLayouts(all);
            } catch (e) {
                console.error("Keyboard Load Error:", e);
            }
        };
        loadKb();
    }, [selectedKeyboard, libraryVersion]);

    const selectKeyboard = (name: string) => {
        setSelectedKeyboard(name);
        localStorage.setItem("last_keyboard", name);
        setLibraryVersion(v => v + 1); // Trigger reload
    };

    const selectCorpus = (filename: string) => {
        setSelectedCorpus(filename);
        localStorage.setItem("last_corpus", filename);
        setLibraryVersion(v => v + 1); // Trigger reload
    };

    const saveUserLayout = async (name: string, layout: string) => {
        await invoke("cmd_save_user_layout", { keyboardId: selectedKeyboard, name, layout });
        // Refresh local cache
        const all = await invoke<Record<string, string>>("cmd_get_all_layouts_scoped", { keyboardId: selectedKeyboard });
        setAvailableLayouts(all);
    };

    const deleteUserLayout = async (name: string) => {
        await invoke("cmd_delete_user_layout", { keyboardId: selectedKeyboard, name });
        const all = await invoke<Record<string, string>>("cmd_get_all_layouts_scoped", { keyboardId: selectedKeyboard });
        setAvailableLayouts(all);
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