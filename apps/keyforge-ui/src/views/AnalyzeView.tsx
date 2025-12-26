import { useKeyboard } from "../context/KeyboardContext";
import { useAnalysis } from "../context/AnalysisContext";
import { Inspector } from "../components/Inspector";
import { KeyboardMap, MapMode } from "../components/KeyboardMap";
import { ExportModal } from "../components/modals/ExportModal";
import { toDisplayString, fromDisplayString } from "../utils";
import { RefreshCw, Activity, Flame, ArrowRightLeft } from "lucide-react";
import { Button } from "../components/ui/Button";
import { useState, useMemo } from "react";

interface Props {
  isSyncing: boolean;
  onSync: () => void;
  localWorkerEnabled: boolean;
  toggleWorker: (b: boolean) => void;
  pinnedKeys: string;
  setPinnedKeys: (s: string) => void;
}

export function AnalyzeView({
  isSyncing,
  onSync,
  localWorkerEnabled,
  toggleWorker,
  pinnedKeys,
  setPinnedKeys,
}: Props) {
  const { activeResult, referenceResult } = useAnalysis();
  const { layoutName, layoutString, selectedKeyboard, availableLayouts } =
    useKeyboard();

  const [mapMode, setMapMode] = useState<MapMode>("frequency");
  const [showDiff, setShowDiff] = useState(false);
  const [isExportOpen, setIsExportOpen] = useState(false);

  // State for keys highlighted by Smart Assist
  const [highlightedKeys, setHighlightedKeys] = useState<Set<string>>(
    new Set(),
  );

  const ghostString =
    showDiff && referenceResult ? availableLayouts["Qwerty"] || "" : "";

  const displayHeatmap = useMemo(() => {
    if (showDiff && activeResult && referenceResult) {
      const activeMap =
        mapMode === "frequency"
          ? activeResult.heatmap
          : activeResult.penaltyMap;
      const refMap =
        mapMode === "frequency"
          ? referenceResult.heatmap
          : referenceResult.penaltyMap;

      if (!activeMap || !refMap) return [];

      return activeMap.map((val, i) => {
        const refVal = refMap[i] || 0;
        return val - refVal;
      });
    } else {
      return mapMode === "frequency"
        ? activeResult?.heatmap
        : activeResult?.penaltyMap;
    }
  }, [activeResult, referenceResult, mapMode, showDiff]);

  const handleSuggestionHover = (indices: number[] | null) => {
    if (!indices || !activeResult) {
      setHighlightedKeys(new Set());
      return;
    }

    const newSet = new Set<string>();
    indices.forEach((idx) => {
      const key = activeResult.geometry.keys[idx];
      if (key && key.id) newSet.add(key.id);
    });
    setHighlightedKeys(newSet);
  };

  return (
    <>
      <div className="flex-1 flex flex-col min-w-0 relative bg-[radial-gradient(ellipse_at_center,_var(--tw-gradient-stops))] from-slate-900/50 to-[#0B0F19]">
        <div className="h-14 flex items-center px-6 border-b border-slate-800/50 justify-between bg-[#0B0F19]/90 backdrop-blur z-10">
          <div className="flex items-center gap-4">
            <h2 className="text-lg font-black text-white tracking-tight">
              {layoutName}
            </h2>
            <span className="text-[10px] px-1.5 py-0.5 rounded bg-slate-800 text-slate-400 font-mono border border-slate-700/50">
              {selectedKeyboard}
            </span>

            <div className="flex bg-slate-900 rounded-lg p-0.5 border border-slate-800">
              <button
                onClick={() => setMapMode("frequency")}
                className={`flex items-center gap-1 px-3 py-1 rounded text-[10px] font-bold transition-all ${mapMode === "frequency" ? "bg-slate-700 text-blue-400 shadow-sm" : "text-slate-500 hover:text-slate-300"}`}
              >
                <Activity size={12} /> Usage
              </button>
              <button
                onClick={() => setMapMode("penalty")}
                className={`flex items-center gap-1 px-3 py-1 rounded text-[10px] font-bold transition-all ${mapMode === "penalty" ? "bg-slate-700 text-red-400 shadow-sm" : "text-slate-500 hover:text-slate-300"}`}
              >
                <Flame size={12} /> Effort
              </button>
            </div>

            {showDiff && (
              <div className="flex items-center gap-2 px-3 py-1 bg-slate-800/50 rounded border border-slate-700 text-[10px] text-slate-300 animate-in fade-in">
                <ArrowRightLeft size={12} />
                <span>Diff Mode</span>
              </div>
            )}
          </div>

          <Button
            size="icon"
            variant="ghost"
            onClick={onSync}
            isLoading={isSyncing}
            icon={<RefreshCw size={18} />}
          />
        </div>

        <div className="flex-1 p-8 flex flex-col items-center justify-center">
          <KeyboardMap
            geometry={activeResult?.geometry}
            layoutString={toDisplayString(fromDisplayString(layoutString))}
            ghostLayoutString={
              ghostString
                ? toDisplayString(fromDisplayString(ghostString))
                : undefined
            }
            heatmap={displayHeatmap}
            mode={showDiff ? "diff" : mapMode}
            activeKeyIds={highlightedKeys}
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
        onExportRequest={() => setIsExportOpen(true)}
        showDiff={showDiff}
        setShowDiff={setShowDiff}
        onSuggestionHover={handleSuggestionHover}
      />

      <ExportModal
        isOpen={isExportOpen}
        onClose={() => setIsExportOpen(false)}
        layoutName={layoutName}
        layoutString={layoutString}
      />
    </>
  );
}
