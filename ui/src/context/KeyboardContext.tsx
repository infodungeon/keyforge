import { ReactNode } from "react";
import { fromDisplayString } from "../utils";
import { LibraryProvider, useLibrary } from "./LibraryContext";
import { SessionProvider, useSession } from "./SessionContext.tsx";

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
// This keeps all existing components working without modification
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
        // Session automatically reacts via useEffect
    };

    const selectCorpus = async (name: string) => {
        lib.selectCorpus(name);
        // Session automatically reacts via useEffect
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