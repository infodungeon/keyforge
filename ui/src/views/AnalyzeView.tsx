// ===== keyforge/ui/src/views/AnalyzeView.tsx =====
import { useKeyboard } from "../context/KeyboardContext";
import { Inspector } from "../components/Inspector";
import { KeyboardMap, MapMode } from "../components/KeyboardMap";
import { toDisplayString, fromDisplayString } from "../utils";
import { RefreshCw, Activity, Flame } from "lucide-react";
import { Button } from "../components/ui/Button";
import { useState } from "react";

interface Props {
    isSyncing: boolean;
    onSync: () => void;
    localWorkerEnabled: boolean;
    toggleWorker: (b: boolean) => void;
    pinnedKeys: string;
    setPinnedKeys: (s: string) => void;
}

export function AnalyzeView({ isSyncing, onSync, localWorkerEnabled, toggleWorker, pinnedKeys, setPinnedKeys }: Props) {
    const {
        activeResult, layoutName, layoutString, selectedKeyboard
    } = useKeyboard();

    const [mapMode, setMapMode] = useState<MapMode>('frequency');

    return (
        <>
            <div className="flex-1 flex flex-col min-w-0 relative bg-[radial-gradient(ellipse_at_center,_var(--tw-gradient-stops))] from-slate-900/50 to-[#0B0F19]">
                {/* Header */}
                <div className="h-14 flex items-center px-6 border-b border-slate-800/50 justify-between bg-[#0B0F19]/90 backdrop-blur z-10">
                    <div className="flex items-center gap-4">
                        <h2 className="text-lg font-black text-white tracking-tight">{layoutName}</h2>
                        <span className="text-[10px] px-1.5 py-0.5 rounded bg-slate-800 text-slate-400 font-mono border border-slate-700/50">
                            {selectedKeyboard}
                        </span>

                        {/* Visualization Toggles */}
                        <div className="flex bg-slate-900 rounded-lg p-0.5 border border-slate-800">
                            <button
                                onClick={() => setMapMode('frequency')}
                                className={`flex items-center gap-1 px-3 py-1 rounded text-[10px] font-bold transition-all ${mapMode === 'frequency' ? 'bg-slate-700 text-blue-400 shadow-sm' : 'text-slate-500 hover:text-slate-300'}`}
                            >
                                <Activity size={12} /> Usage
                            </button>
                            <button
                                onClick={() => setMapMode('penalty')}
                                className={`flex items-center gap-1 px-3 py-1 rounded text-[10px] font-bold transition-all ${mapMode === 'penalty' ? 'bg-slate-700 text-red-400 shadow-sm' : 'text-slate-500 hover:text-slate-300'}`}
                            >
                                <Flame size={12} /> Effort
                            </button>
                        </div>
                    </div>

                    <Button size="icon" variant="ghost" onClick={onSync} isLoading={isSyncing} icon={<RefreshCw size={18} />} />
                </div>

                {/* Map */}
                <div className="flex-1 p-8 flex flex-col items-center justify-center">
                    <KeyboardMap
                        geometry={activeResult?.geometry}
                        layoutString={toDisplayString(fromDisplayString(layoutString))}
                        heatmap={mapMode === 'frequency' ? activeResult?.heatmap : activeResult?.penaltyMap}
                        mode={mapMode}
                        className="w-full h-full max-w-4xl"
                    />
                </div>
            </div>

            <Inspector
                mode="analyze"
                localWorkerEnabled={localWorkerEnabled}
                toggleWorker={toggleWorker}
                pinnedKeys={pinnedKeys}
                setPinnedKeys={setPinnedKeys}
            />
        </>
    );
}