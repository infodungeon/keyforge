import { useKeyboard } from "../context/KeyboardContext";
import { Inspector } from "../components/Inspector";
import { KeyboardMap } from "../components/KeyboardMap";
import { calculateStats, toDisplayString, fromDisplayString } from "../utils";
import { RefreshCw } from "lucide-react";
import { Button } from "../components/ui/Button";

interface Props {
    isSyncing: boolean;
    onSync: () => void;
    // Pass-throughs
    localWorkerEnabled: boolean;
    toggleWorker: (b: boolean) => void;
    pinnedKeys: string;
    setPinnedKeys: (s: string) => void;
}

export function AnalyzeView({
    isSyncing, onSync,
    localWorkerEnabled, toggleWorker, pinnedKeys, setPinnedKeys
}: Props) {
    const {
        activeResult, referenceResult, layoutName, layoutString,
        selectedKeyboard, keyboards, selectKeyboard,
        availableLayouts, loadLayoutPreset, setLayoutName,
        weights, searchParams, setWeights, setSearchParams,
        activeJobId, saveUserLayout, deleteUserLayout, standardLayouts
    } = useKeyboard();

    const derivedStats = (activeResult?.geometry && activeResult?.heatmap)
        ? calculateStats(activeResult.geometry, activeResult.heatmap) : null;

    const isStandard = standardLayouts.includes(layoutName);
    const isCustom = layoutName === "Custom";

    return (
        <>
            <div className="flex-1 flex flex-col min-w-0 relative bg-[radial-gradient(ellipse_at_center,_var(--tw-gradient-stops))] from-slate-900/50 to-[#0B0F19]">
                <div className="h-14 flex items-center px-6 border-b border-slate-800/50 justify-between bg-[#0B0F19]/90 backdrop-blur z-10">
                    <div className="flex items-center gap-2">
                        <h2 className="text-lg font-black text-white tracking-tight">{layoutName}</h2>
                        <span className="text-[10px] px-1.5 py-0.5 rounded bg-slate-800 text-slate-400 font-mono border border-slate-700/50">
                            {selectedKeyboard}
                        </span>
                    </div>
                    <Button size="icon" variant="ghost" onClick={onSync} isLoading={isSyncing} icon={<RefreshCw size={18} />} />
                </div>

                <div className="flex-1 p-8 flex flex-col items-center justify-center">
                    <KeyboardMap
                        geometry={activeResult?.geometry}
                        layoutString={toDisplayString(fromDisplayString(layoutString))}
                        heatmap={activeResult?.heatmap}
                        className="w-full h-full max-w-4xl"
                    />
                </div>
            </div>

            <Inspector
                mode="analyze"
                keyboards={keyboards} selectedKeyboard={selectedKeyboard} setSelectedKeyboard={selectKeyboard}
                availableLayouts={availableLayouts} layoutName={layoutName} loadLayoutPreset={loadLayoutPreset} setLayoutName={setLayoutName}
                activeResult={activeResult} referenceResult={referenceResult} derivedStats={derivedStats}
                weights={weights} searchParams={searchParams} activeJobId={activeJobId} pinnedKeys={pinnedKeys} isInitializing={false}
                setWeights={setWeights} setSearchParams={setSearchParams} setPinnedKeys={setPinnedKeys}
                handleDispatch={() => { }} handleStop={() => { }} handleImport={() => { }} handleExport={() => { }} runLocalValidation={() => { }}

                // Fixed: Removed layoutString, showComparison props
                showDiff={false} setShowDiff={() => { }}
                localWorkerEnabled={localWorkerEnabled} toggleWorker={toggleWorker}
                isStandard={isStandard} isCustom={isCustom}
                onSave={() => {
                    const name = prompt("Name your layout:", layoutName === "Custom" ? "My Layout" : layoutName);
                    if (name) saveUserLayout(name);
                }}
                onDelete={() => {
                    if (confirm(`Delete ${layoutName}?`)) deleteUserLayout(layoutName);
                }}
                onSubmit={() => { }}
            />
        </>
    );
}