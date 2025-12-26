import { useState } from "react";
import { useKeyboard } from "../context/KeyboardContext";
import { useAnalysis } from "../context/AnalysisContext";
import { useArena } from "../context/ArenaContext";
import { useToast } from "../context/ToastContext";
import { useBackend } from "../context/BackendContext";
import { useSystem } from "../context/SystemContext";
import { AppMode } from "../types";
import { calculateStats } from "../utils";
import { ContextControls } from "./ContextControls";
import { Button } from "./ui/Button";
import { SmartSuggestions } from "./SmartSuggestions";
import {
  BarChart2,
  Sliders,
  Settings as SettingsIcon,
  Play,
  Square,
  Save,
  Trash2,
  Send,
  Activity,
  FileCode,
} from "lucide-react";

import { AnalyzePanel } from "./panels/AnalyzePanel";
import { OptimizePanel } from "./panels/OptimizePanel";
import { ArenaPanel } from "./panels/ArenaPanel";

interface Props {
  mode: AppMode;
  onDispatch?: () => void;
  onStop?: () => void;
  localWorkerEnabled?: boolean;
  toggleWorker?: (b: boolean) => void;
  pinnedKeys?: string;
  setPinnedKeys?: (s: string) => void;
  onExportRequest?: () => void;
  showDiff?: boolean;
  setShowDiff?: (b: boolean) => void;
  onSuggestionHover?: (indices: number[] | null) => void;
}

export function Inspector({
  mode,
  onDispatch,
  onStop,
  localWorkerEnabled,
  toggleWorker,
  pinnedKeys,
  setPinnedKeys,
  onExportRequest,
  showDiff: controlledShowDiff,
  setShowDiff: controlledSetShowDiff,
  onSuggestionHover,
}: Props) {
  const { activeResult, referenceResult } = useAnalysis();
  const {
    layoutName,
    activeJobId,
    layoutString,
    weights,
    searchParams,
    setWeights,
    setSearchParams,
    standardLayouts,
    saveUserLayout,
    deleteUserLayout,
  } = useKeyboard();

  const arena = mode === "arena" ? useArena() : null;
  const { addToast } = useToast();
  const { hiveUrl, hiveSecret } = useSystem();
  const backend = useBackend();
  const [isPosting, setIsPosting] = useState(false);

  const [localShowDiff, setLocalShowDiff] = useState(false);
  const showDiff = controlledShowDiff ?? localShowDiff;
  const setShowDiff = controlledSetShowDiff ?? setLocalShowDiff;

  const derivedStats =
    activeResult?.geometry && activeResult?.heatmap
      ? calculateStats(activeResult.geometry, activeResult.heatmap)
      : null;

  const isStandard = standardLayouts.includes(layoutName);

  const { title: PanelTitle, icon: PanelIcon } =
    mode === "analyze"
      ? { title: "Analyze", icon: BarChart2 }
      : mode === "optimize"
        ? { title: "Optimize", icon: Sliders }
        : mode === "arena"
          ? { title: "Arena Stats", icon: Activity }
          : { title: "Configuration", icon: SettingsIcon };

  const handleSave = () => {
    const name = prompt(
      "Name your layout:",
      layoutName === "Custom" ? "My Layout" : layoutName,
    );
    if (name) saveUserLayout(name);
  };

  const handleDelete = () => {
    if (confirm(`Delete ${layoutName}?`)) deleteUserLayout(layoutName);
  };

  const handlePost = async () => {
    if (!layoutString) return;
    const savedAuthor = localStorage.getItem("keyforge_author") || "";
    const author = prompt("Enter your name/handle:", savedAuthor);
    if (!author) return;
    localStorage.setItem("keyforge_author", author);
    const name =
      layoutName === "Custom"
        ? prompt("Name this layout:", "Untitled")
        : layoutName;
    if (!name) return;

    setIsPosting(true);
    try {
      await backend.submitUserLayout(
        hiveUrl,
        hiveSecret,
        name,
        layoutString,
        author,
      );
      addToast("success", "Layout submitted to Hive!");
    } catch (e) {
      addToast("error", `Submission failed: ${e}`);
    } finally {
      setIsPosting(false);
    }
  };

  return (
    <div className="w-80 bg-slate-900 border-l border-slate-800 flex flex-col overflow-hidden shrink-0">
      <div className="p-4 border-b border-slate-800 flex justify-between items-center bg-slate-950/30">
        <h3 className="text-xs font-bold text-slate-400 uppercase flex items-center gap-2">
          <PanelIcon size={14} /> {PanelTitle}
        </h3>
        {mode === "analyze" && (
          <div className="flex gap-1">
            <button
              onClick={onExportRequest}
              title="Export Firmware"
              className="p-1.5 hover:bg-slate-800 rounded text-slate-500 hover:text-white transition-colors"
            >
              <FileCode size={14} />
            </button>
          </div>
        )}
      </div>

      <ContextControls
        disabled={!!activeJobId || (mode === "arena" && !arena?.isFinished)}
      />

      <div className="flex-1 overflow-y-auto p-4 custom-scrollbar">
        {mode === "analyze" && (
          <SmartSuggestions onHover={onSuggestionHover || (() => {})} />
        )}

        {mode === "analyze" && (
          <AnalyzePanel
            activeResult={activeResult}
            referenceResult={referenceResult}
            derivedStats={derivedStats}
            showDiff={showDiff}
            setShowDiff={setShowDiff}
          />
        )}

        {mode === "optimize" &&
          weights &&
          searchParams &&
          setPinnedKeys &&
          toggleWorker && (
            <OptimizePanel
              weights={weights}
              searchParams={searchParams}
              pinnedKeys={pinnedKeys || ""}
              localWorkerEnabled={localWorkerEnabled || false}
              setWeights={setWeights}
              setSearchParams={setSearchParams}
              setPinnedKeys={setPinnedKeys}
              toggleWorker={toggleWorker}
            />
          )}

        {mode === "arena" && <ArenaPanel />}
      </div>

      <div className="p-4 border-t border-slate-800 bg-slate-900/80">
        {mode === "optimize" ? (
          !activeJobId ? (
            <Button
              variant="optimize"
              className="w-full"
              onClick={onDispatch}
              icon={<Play size={16} />}
            >
              START OPTIMIZATION
            </Button>
          ) : (
            <Button
              variant="danger"
              className="w-full"
              onClick={onStop}
              icon={<Square size={16} />}
            >
              STOP JOB
            </Button>
          )
        ) : mode !== "arena" ? (
          <div className="grid grid-cols-3 gap-2">
            <Button
              variant="secondary"
              size="sm"
              onClick={handleSave}
              icon={<Save size={14} />}
            >
              Save
            </Button>
            <Button
              variant="secondary"
              size="sm"
              onClick={handleDelete}
              disabled={isStandard}
              icon={<Trash2 size={14} />}
            >
              Del
            </Button>
            <Button
              variant="secondary"
              size="sm"
              onClick={handlePost}
              isLoading={isPosting}
              icon={<Send size={14} />}
            >
              Post
            </Button>
          </div>
        ) : null}
      </div>
    </div>
  );
}
