import { AppMode, ValidationResult, ScoringWeights, SearchParams } from "../types";
import { DerivedStats } from "../utils";
import { ContextControls } from "./ContextControls";
import { Button } from "./ui/Button";
import { BarChart2, Sliders, Settings as SettingsIcon, Download, Upload, Play, Square, Save, Trash2, Send } from "lucide-react";

// Sub-panels
import { AnalyzePanel } from "./panels/AnalyzePanel";
import { OptimizePanel } from "./panels/OptimizePanel";

interface Props {
    mode: AppMode;
    // Context Data
    keyboards: string[];
    selectedKeyboard: string;
    setSelectedKeyboard: (s: string) => void;
    availableLayouts: Record<string, string>;
    layoutName: string;
    setLayoutName: (s: string) => void;
    loadLayoutPreset: (n: string) => void;
    activeResult: ValidationResult | null;
    referenceResult: ValidationResult | null;
    derivedStats: DerivedStats | null;
    weights: ScoringWeights | null;
    searchParams: SearchParams | null;
    activeJobId: string | null;
    pinnedKeys: string;
    isInitializing: boolean;
    // Actions
    setWeights: (w: ScoringWeights) => void;
    setSearchParams: (s: SearchParams) => void;
    setPinnedKeys: (s: string) => void;
    handleDispatch: () => void;
    handleStop: () => void;
    handleImport: () => void;
    handleExport: () => void;
    showDiff: boolean;
    setShowDiff: (b: boolean) => void;
    localWorkerEnabled: boolean;
    toggleWorker: (b: boolean) => void;
    isStandard: boolean;
    onSave: () => void;
    onDelete: () => void;
    onSubmit: () => void;
}

export function Inspector(props: Props) {
    const { mode, activeJobId } = props;

    const { title: PanelTitle, icon: PanelIcon } =
        mode === 'analyze' ? { title: 'Analyze', icon: BarChart2 } :
            mode === 'optimize' ? { title: 'Optimize', icon: Sliders } :
                { title: 'Configuration', icon: SettingsIcon };

    return (
        <div className="w-80 bg-slate-900 border-l border-slate-800 flex flex-col overflow-hidden shrink-0">
            {/* Header */}
            <div className="p-4 border-b border-slate-800 flex justify-between items-center bg-slate-950/30">
                <h3 className="text-xs font-bold text-slate-400 uppercase flex items-center gap-2">
                    <PanelIcon size={14} /> {PanelTitle}
                </h3>
                {mode === 'analyze' && (
                    <div className="flex gap-1">
                        <button onClick={props.handleImport} title="Import" className="p-1.5 hover:bg-slate-800 rounded text-slate-500 hover:text-white transition-colors"> <Download size={14} /> </button>
                        <button onClick={props.handleExport} title="Export" className="p-1.5 hover:bg-slate-800 rounded text-slate-500 hover:text-white transition-colors"> <Upload size={14} /> </button>
                    </div>
                )}
            </div>

            {/* Common Context Controls */}
            <ContextControls
                keyboards={props.keyboards}
                selectedKeyboard={props.selectedKeyboard}
                onSelectKeyboard={props.setSelectedKeyboard}
                availableLayouts={props.availableLayouts}
                layoutName={props.layoutName}
                onSelectLayout={props.loadLayoutPreset}
                disabled={!!activeJobId || props.isInitializing}
            />

            {/* Scrollable Content Area */}
            <div className="flex-1 overflow-y-auto p-4 custom-scrollbar">
                {mode === 'analyze' && (
                    <AnalyzePanel
                        activeResult={props.activeResult}
                        referenceResult={props.referenceResult}
                        derivedStats={props.derivedStats}
                        showDiff={props.showDiff}
                        setShowDiff={props.setShowDiff}
                    />
                )}

                {mode === 'optimize' && (
                    <OptimizePanel
                        weights={props.weights}
                        searchParams={props.searchParams}
                        pinnedKeys={props.pinnedKeys}
                        localWorkerEnabled={props.localWorkerEnabled}
                        setWeights={props.setWeights}
                        setSearchParams={props.setSearchParams}
                        setPinnedKeys={props.setPinnedKeys}
                        toggleWorker={props.toggleWorker}
                    />
                )}
            </div>

            {/* Footer / Actions */}
            <div className="p-4 border-t border-slate-800 bg-slate-900/80">
                {mode === 'optimize' ? (
                    !activeJobId ? (
                        <Button variant="secondary" className="w-full" onClick={props.handleDispatch} icon={<Play size={16} />}>
                            START JOB
                        </Button>
                    ) : (
                        <Button variant="danger" className="w-full" onClick={props.handleStop} icon={<Square size={16} />}>
                            STOP JOB
                        </Button>
                    )
                ) : (
                    <div className="grid grid-cols-3 gap-2">
                        <Button variant="secondary" size="sm" onClick={props.onSave} icon={<Save size={14} />}>Save</Button>
                        <Button variant="secondary" size="sm" onClick={props.onDelete} disabled={props.isStandard} icon={<Trash2 size={14} />}>Del</Button>
                        <Button variant="secondary" size="sm" onClick={props.onSubmit} icon={<Send size={14} />}>Post</Button>
                    </div>
                )}
            </div>
        </div>
    );
}