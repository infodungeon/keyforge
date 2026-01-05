import { useKeyboard } from "../context/KeyboardContext";
import { useAnalysis } from "../context/AnalysisContext";
import { Inspector } from "../components/Inspector";
import { KeyboardMap, MapMode } from "../components/KeyboardMap";
import { ExportModal } from "../components/modals/ExportModal";
import { toDisplayString, fromDisplayString, calculateStats } from "../utils";
import { adjustHeatmap, SpaceHandPreference } from "../services/stats";
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
  const [includeThumbs, setIncludeThumbs] = useState(true);
  const [spaceHand, setSpaceHand] = useState<SpaceHandPreference>("bilateral");

  // State for keys highlighted by Smart Assist
  const [highlightedKeys, setHighlightedKeys] = useState<Set<string>>(
    new Set(),
  );

  const ghostString =
    showDiff && referenceResult ? availableLayouts["Qwerty"] || "" : "";

  // 1. Calculate Adjusted Heatmaps (Redistribute Space)
  const adjustedActiveMap = useMemo(() => {
    if (!activeResult?.heatmap || !activeResult.geometry) return [];
    return adjustHeatmap(
        activeResult.geometry, 
        activeResult.heatmap, 
        layoutString.trim().split(/\s+/), 
        spaceHand
    );
  }, [activeResult, layoutString, spaceHand]);

  const adjustedPenaltyMap = useMemo(() => {
    if (!activeResult?.penalty_map || !activeResult.geometry) return [];
    return adjustHeatmap(
        activeResult.geometry,
        activeResult.penalty_map,
        layoutString.trim().split(/\s+/),
        spaceHand
    );
  }, [activeResult, layoutString, spaceHand]);

  const adjustedRefMap = useMemo(() => {
    if (!referenceResult?.heatmap || !referenceResult.geometry) return [];
    // For reference, we assume bilateral for now
    return referenceResult.heatmap; 
  }, [referenceResult]);

  // 2. Calculate Display Heatmap (Diff or Single)
  const displayHeatmap = useMemo(() => {
    // Helper to filter thumbs if needed
    const filterValue = (val: number, idx: number, geometry: any) => {
      if (includeThumbs) return val;
      if (!geometry || !geometry.keys[idx]) return val;
      // Finger 0 is thumb
      const finger = geometry.keys[idx].finger;
      return finger === 0 ? 0 : val;
    };

    if (showDiff && activeResult && referenceResult) {
      const activeMap = mapMode === "frequency" ? adjustedActiveMap : adjustedPenaltyMap;
      const refMap = mapMode === "frequency" ? adjustedRefMap : referenceResult.penalty_map;

      if (!activeMap || !refMap) return [];

      return activeMap.map((val, i) => {
        const v1 = filterValue(val, i, activeResult.geometry);
        const v2 = filterValue(refMap[i] || 0, i, referenceResult.geometry);
        return v1 - v2;
      });
    } else {
      const sourceMap = mapMode === "frequency" ? adjustedActiveMap : adjustedPenaltyMap;
      if (!sourceMap) return undefined;
      return sourceMap.map((val, i) => filterValue(val, i, activeResult?.geometry));
    }
  }, [activeResult, referenceResult, mapMode, showDiff, includeThumbs, adjustedActiveMap, adjustedPenaltyMap, adjustedRefMap]);

  // 3. Calculate Stats based on Adjusted Heatmap
  const derivedStats = useMemo(() => {
      if (!activeResult?.geometry) return null;

      const sourceMap = mapMode === "penalty" ? adjustedPenaltyMap : adjustedActiveMap;
      if (sourceMap.length === 0) return null;

      return calculateStats(
        activeResult.geometry, 
        sourceMap, 
        layoutString.trim().split(/\s+/), 
        spaceHand, 
        includeThumbs
      );
  }, [activeResult, adjustedActiveMap, adjustedPenaltyMap, mapMode, layoutString, spaceHand, includeThumbs]);

  const handleSuggestionHover = (indices: number[] | null) => {
    if (!indices || !activeResult) {
      setHighlightedKeys(new Set());
      return;
    }

    const newSet = new Set<string>();
    indices.forEach((idx) => {
      const key = activeResult.geometry.keys[idx];
      if (key && key.label) newSet.add(key.label);
    });
    setHighlightedKeys(newSet);
  };

  return (
    <>
      <div className="flex-1 flex flex-col min-w-0 relative bg-[radial-gradient(ellipse_at_center,_var(--tw-gradient-stops))] from-slate-900/50 to-[#0B0F19]">
        <div className="h-14 flex items-center px-6 border-b border-slate-800/50 justify-between bg-[#0B0F19]/90 backdrop-blur z-10">
          <div className="flex items-center gap-4">
            <h2 className="text-2xl font-black text-white tracking-tight">
              {selectedKeyboard}
            </h2>
            <span className="text-xs font-bold text-slate-500 uppercase tracking-wider">
              {layoutName}
            </span>

            <div className="flex bg-slate-900 rounded-lg p-0.5 border border-slate-800">
              <button
                onClick={() => setMapMode("frequency")}
                className={`flex items-center gap-1 px-3 py-1 rounded text-[10px] font-bold transition-all ${mapMode === "frequency" ? "bg-slate-700 text-red-400 shadow-sm" : "text-slate-500 hover:text-slate-300"}`}
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
        includeThumbs={includeThumbs}
        setIncludeThumbs={setIncludeThumbs}
        spaceHand={spaceHand}
        setSpaceHand={setSpaceHand}
        derivedStats={derivedStats}
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
