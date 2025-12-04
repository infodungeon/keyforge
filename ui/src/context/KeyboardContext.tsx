// ===== keyforge/ui/src/context/KeyboardContext.tsx =====
import { ReactNode } from "react";
import { fromDisplayString } from "../utils";
import { LibraryProvider, useLibrary } from "./LibraryContext";
import { SessionProvider, useSession } from "./SessionContext";

// Export the Combined Provider
export function KeyboardProvider({ children }: { children: ReactNode }) {
    return (
        <LibraryProvider>
            <SessionProvider>
                {children}
            </SessionProvider>
        </LibraryProvider>
    );
}

// Export the Hook that merges both contexts
export const useKeyboard = () => {
    const lib = useLibrary();
    const sess = useSession();

    // Merge Actions
    const saveUserLayout = async (name: string) => {
        const standardized = fromDisplayString(sess.layoutString);
        await lib.saveUserLayout(name, standardized);
        sess.setLayoutName(name);
    };

    const deleteUserLayout = async (name: string) => {
        await lib.deleteUserLayout(name);
        sess.setLayoutName("Custom");
    };

    const refreshData = async () => {
        await lib.refreshLibrary();
    };

    const selectKeyboard = async (name: string) => {
        lib.selectKeyboard(name);
    };

    const selectCorpus = async (name: string) => {
        lib.selectCorpus(name);
    };

    const selectCostMatrix = async (name: string) => {
        lib.selectCostMatrix(name);
    };

    return {
        // Library State
        weights: lib.weights,
        setWeights: lib.setWeights,
        searchParams: lib.searchParams,
        setSearchParams: lib.setSearchParams,
        keyboards: lib.keyboards,
        selectedKeyboard: lib.selectedKeyboard,
        selectKeyboard,
        corpora: lib.corpora,
        selectedCorpus: lib.selectedCorpus,
        selectCorpus,

        // ADDED: Cost Matrix State
        costMatrices: lib.costMatrices,
        selectedCostMatrix: lib.selectedCostMatrix,
        selectCostMatrix,

        availableLayouts: lib.availableLayouts,
        standardLayouts: lib.standardLayouts,

        // Session State
        layoutName: sess.layoutName,
        layoutString: sess.layoutString,
        setLayoutName: sess.setLayoutName,
        updateLayoutString: sess.updateLayoutString,
        loadLayoutPreset: sess.loadLayoutPreset,
        activeResult: sess.activeResult,
        referenceResult: sess.referenceResult,
        isValidating: sess.isValidating,
        activeJobId: sess.activeJobId,
        startJob: sess.startJob,
        stopJob: sess.stopJob,
        selectedKeyIndex: sess.selectedKeyIndex,
        setSelectedKeyIndex: sess.setSelectedKeyIndex,

        // Merged Actions
        saveUserLayout,
        deleteUserLayout,
        refreshData
    };
};