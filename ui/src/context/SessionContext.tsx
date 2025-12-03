import { createContext, useContext, useState, useEffect, ReactNode, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ValidationResult, ScoringWeights } from "../types";
import { formatForDisplay, fromDisplayString } from "../utils";
import { useLibrary } from "./LibraryContext";

interface SessionContextType {
    layoutName: string;
    layoutString: string;
    setLayoutName: (n: string) => void;
    updateLayoutString: (s: string) => void;
    loadLayoutPreset: (name: string) => void;
    
    activeResult: ValidationResult | null;
    referenceResult: ValidationResult | null;
    isValidating: boolean;
    
    activeJobId: string | null;
    startJob: (id: string) => void;
    stopJob: () => void;
    
    selectedKeyIndex: number | null;
    setSelectedKeyIndex: (i: number | null) => void;
}

const SessionContext = createContext<SessionContextType | undefined>(undefined);

export function SessionProvider({ children }: { children: ReactNode }) {
    const { 
        selectedKeyboard, selectedCorpus, libraryVersion, 
        availableLayouts, standardLayouts, weights,
        saveUserLayout: libSave, deleteUserLayout: libDelete
    } = useLibrary();

    const [layoutName, setLayoutName] = useState("Custom");
    const [layoutString, setLayoutString] = useState("");
    const [selectedKeyIndex, setSelectedKeyIndex] = useState<number | null>(null);
    
    const [activeResult, setActiveResult] = useState<ValidationResult | null>(null);
    const [referenceResult, setReferenceResult] = useState<ValidationResult | null>(null);
    const [isValidating, setIsValidating] = useState(false);
    const [activeJobId, setActiveJobId] = useState<string | null>(null);

    // --- Validation Logic ---
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

    // --- Synchronization with Library ---
    useEffect(() => {
        if (!selectedKeyboard || !selectedCorpus) return;

        const syncSession = async () => {
            try {
                // 1. Tell backend to load the dataset (Cost + Ngrams + Geometry)
                await invoke("cmd_load_dataset", { keyboardName: selectedKeyboard, corpusFilename: selectedCorpus });

                // 2. Determine initial layout state
                const preferred = "Qwerty";
                const defName = availableLayouts[preferred] ? preferred : Object.keys(availableLayouts)[0] || "Custom";
                const qmkStr = availableLayouts[defName] || "";

                setLayoutName(defName);
                setLayoutString(formatForDisplay(qmkStr));
                setSelectedKeyIndex(null);

                // 3. Validate Initial & Reference
                if (qmkStr) {
                    // Reference (Qwerty usually)
                    if (availableLayouts["Qwerty"]) {
                        const ref = await invoke<ValidationResult>("cmd_validate_layout", { layoutStr: availableLayouts["Qwerty"], weights: null });
                        setReferenceResult(ref);
                    }
                    // Active
                    runValidation(defName, qmkStr, weights);
                }
            } catch (e) {
                console.error("Session Sync Failed:", e);
            }
        };

        syncSession();
    }, [selectedKeyboard, selectedCorpus, libraryVersion, availableLayouts, weights]);

    // --- Actions ---

    const updateLayoutString = (val: string) => {
        if (standardLayouts.includes(layoutName)) {
            setLayoutName("Custom");
        }
        setLayoutString(val);
        // Debounce validation could go here, for now we validate on 'update' logic inside components or effects
        // But typically the UI calls runValidation manually or we trigger it here:
        runValidation(standardLayouts.includes(layoutName) ? "Custom" : layoutName, fromDisplayString(val), weights);
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

    const startJob = (id: string) => setActiveJobId(id);
    const stopJob = () => {
        setActiveJobId(null);
        // Optionally kill backend command
        invoke("cmd_stop_search").catch(console.error);
    };

    return (
        <SessionContext.Provider value={{
            layoutName, layoutString, setLayoutName, updateLayoutString, loadLayoutPreset,
            activeResult, referenceResult, isValidating,
            activeJobId, startJob, stopJob,
            selectedKeyIndex, setSelectedKeyIndex
        }}>
            {children}
        </SessionContext.Provider>
    );
}

export const useSession = () => {
    const ctx = useContext(SessionContext);
    if (!ctx) throw new Error("useSession must be used within SessionProvider");
    return ctx;
};