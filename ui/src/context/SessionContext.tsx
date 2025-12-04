import { createContext, useContext, useState, useEffect, ReactNode, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ValidationResult, ScoringWeights } from "../types";
import { formatForDisplay, fromDisplayString } from "../utils";
import { useLibrary } from "./LibraryContext";
import { useToast } from "./ToastContext";

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
        availableLayouts, standardLayouts, weights
    } = useLibrary();

    const { addToast } = useToast();

    const [layoutName, setLayoutName] = useState("Custom");
    const [layoutString, setLayoutString] = useState("");
    const [selectedKeyIndex, setSelectedKeyIndex] = useState<number | null>(null);

    const [activeResult, setActiveResult] = useState<ValidationResult | null>(null);
    const [referenceResult, setReferenceResult] = useState<ValidationResult | null>(null);
    const [isValidating, setIsValidating] = useState(false);
    const [activeJobId, setActiveJobId] = useState<string | null>(null);

    // Track if dataset is ready to prevent validation calls on empty state
    const [isDatasetLoaded, setIsDatasetLoaded] = useState(false);

    // --- Validation Logic ---
    const runValidation = useCallback(async (name: string, qmkStr: string, w: ScoringWeights | null) => {
        if (!qmkStr) return;
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

        let mounted = true;

        const syncSession = async () => {
            setIsDatasetLoaded(false);
            try {
                // 1. Tell backend to load the dataset
                // This acquires a WRITE LOCK on the backend.
                await invoke("cmd_load_dataset", { keyboardName: selectedKeyboard, corpusFilename: selectedCorpus });

                if (!mounted) return;
                setIsDatasetLoaded(true);

                // 2. Determine initial layout state
                // Prefer Qwerty as reference, otherwise first available
                const preferred = "Qwerty";
                const defName = availableLayouts[preferred] ? preferred : Object.keys(availableLayouts)[0] || "Custom";
                const qmkStr = availableLayouts[defName] || "";

                setLayoutName(defName);
                setLayoutString(formatForDisplay(qmkStr));
                setSelectedKeyIndex(null);

                // 3. Validate Initial & Reference
                if (qmkStr) {
                    if (availableLayouts["Qwerty"]) {
                        const ref = await invoke<ValidationResult>("cmd_validate_layout", { layoutStr: availableLayouts["Qwerty"], weights: null });
                        if (mounted) setReferenceResult(ref);
                    }
                    // Validate Active
                    if (mounted) runValidation(defName, qmkStr, weights);
                }
            } catch (e) {
                console.error("Session Sync Failed:", e);
                addToast('error', `Failed to load dataset: ${e}`);
            }
        };

        syncSession();

        return () => { mounted = false; };
    }, [selectedKeyboard, selectedCorpus, libraryVersion, availableLayouts, weights]);

    // --- Actions ---

    const updateLayoutString = (val: string) => {
        if (!isDatasetLoaded) return;

        if (standardLayouts.includes(layoutName)) {
            setLayoutName("Custom");
        }
        setLayoutString(val);

        runValidation(standardLayouts.includes(layoutName) ? "Custom" : layoutName, fromDisplayString(val), weights);
    };

    const loadLayoutPreset = (name: string) => {
        if (!isDatasetLoaded) return;

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
        invoke("cmd_stop_search").catch(e => console.error("Stop failed:", e));
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