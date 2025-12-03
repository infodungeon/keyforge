import { useState } from "react";
import { AppMode, ValidationResult, ScoringWeights, SearchParams } from "../types";
import { DerivedStats } from "../utils";
import { StatBox, FingerBar } from "./Charts";
import { OptimizerConfig } from "./OptimizerConfig";
import { ContextControls } from "./ContextControls";
import { Button } from "./ui/Button";
import { Input } from "./ui/Input";
import { Label } from "./ui/Label";
import { Card } from "./ui/Card";
import {
    BarChart2, Sliders, Settings as SettingsIcon,
    Download, Upload, Play, Square,
    ChevronDown, ChevronRight, ArrowRightLeft,
    Save, Trash2, Send
} from "lucide-react";

interface Props {
    mode: AppMode;
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
    setWeights: (w: ScoringWeights) => void;
    setSearchParams: (s: SearchParams) => void;
    setPinnedKeys: (s: string) => void;
    handleDispatch: () => void;
    handleStop: () => void;
    handleImport: () => void;
    handleExport: () => void;
    runLocalValidation: (name: string, str: string) => void;
    showDiff: boolean;
    setShowDiff: (b: boolean) => void;
    localWorkerEnabled: boolean;
    toggleWorker: (b: boolean) => void;
    isStandard: boolean;
    isCustom: boolean;
    onSave: () => void;
    onDelete: () => void;
    onSubmit: () => void;
}

export function Inspector(props: Props) {
    const { mode, activeResult, referenceResult, derivedStats, weights, searchParams, activeJobId } = props;
    const [showAdvanced, setShowAdvanced] = useState(false);

    const { title: PanelTitle, icon: PanelIcon } =
        mode === 'analyze' ? { title: 'Analyze', icon: BarChart2 } :
            mode === 'optimize' ? { title: 'Optimize', icon: Sliders } :
                mode === 'settings' ? { title: 'Settings', icon: SettingsIcon } :
                    { title: 'Configuration', icon: SettingsIcon };

    return (
        <div className="w-80 bg-slate-900 border-l border-slate-800 flex flex-col overflow-hidden shrink-0">
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

            <ContextControls
                keyboards={props.keyboards}
                selectedKeyboard={props.selectedKeyboard}
                onSelectKeyboard={props.setSelectedKeyboard}
                availableLayouts={props.availableLayouts}
                layoutName={props.layoutName}
                onSelectLayout={props.loadLayoutPreset}
                disabled={!!activeJobId || props.isInitializing}
            />

            <div className="flex-1 overflow-y-auto p-4 custom-scrollbar space-y-6">

                {mode === 'analyze' && activeResult && derivedStats && (
                    <>
                        <div className="bg-slate-800/50 rounded-xl p-4 border border-slate-700 relative overflow-hidden group">
                            <div className="text-[10px] text-slate-500 uppercase tracking-widest font-bold mb-1">Total Score</div>
                            <div className="text-4xl font-black text-transparent bg-clip-text bg-gradient-to-r from-blue-400 to-purple-400 font-mono">
                                {activeResult.score.layoutScore.toFixed(0)}
                            </div>
                            {referenceResult && (
                                <div className="mt-3 pt-3 border-t border-slate-700/50 flex items-center justify-between">
                                    <span className="text-[10px] text-slate-400">vs Ref:
                                        <span className={(referenceResult.score.layoutScore - activeResult.score.layoutScore) > 0 ? "text-green-400 ml-1 font-bold" : "text-red-400 ml-1 font-bold"}>
                                            {(activeResult.score.layoutScore - referenceResult.score.layoutScore).toFixed(0)}
                                        </span>
                                    </span>
                                    <button onClick={() => props.setShowDiff(!props.showDiff)}
                                        className={`text-[10px] flex items-center gap-1 px-2 py-1 rounded transition-colors ${props.showDiff ? "bg-blue-500 text-white" : "bg-slate-700/50 text-slate-400"}`}>
                                        <ArrowRightLeft size={10} /> {props.showDiff ? "Active" : "Compare"}
                                    </button>
                                </div>
                            )}
                        </div>

                        <div>
                            <h4 className="text-[10px] font-bold text-slate-500 uppercase mb-3">Balance</h4>
                            <div className="flex gap-1 h-3 rounded-full overflow-hidden mb-1">
                                <div className="bg-blue-500 transition-all" style={{ width: `${derivedStats.handBalance.left}%` }} />
                                <div className="bg-purple-500 transition-all" style={{ width: `${derivedStats.handBalance.right}%` }} />
                            </div>
                            <div className="flex justify-between text-[10px] text-slate-300 mb-4 font-mono font-bold">
                                <span>L: {derivedStats.handBalance.left.toFixed(1)}%</span>
                                <span>R: {derivedStats.handBalance.right.toFixed(1)}%</span>
                            </div>
                        </div>

                        <div>
                            <h4 className="text-[10px] font-bold text-slate-500 uppercase mb-3">Finger Load</h4>
                            <div className="space-y-1">
                                <FingerBar label="Pinky" pct={derivedStats.fingerUsage[4]} color="bg-pink-500" />
                                <FingerBar label="Ring" pct={derivedStats.fingerUsage[3]} color="bg-purple-500" />
                                <FingerBar label="Mid" pct={derivedStats.fingerUsage[2]} color="bg-blue-500" />
                                <FingerBar label="Index" pct={derivedStats.fingerUsage[1]} color="bg-green-500" />
                                <FingerBar label="Thumb" pct={derivedStats.fingerUsage[0]} color="bg-slate-500" />
                            </div>
                        </div>

                        <div>
                            <div className="flex items-center justify-between mb-3">
                                <h4 className="text-[10px] font-bold text-slate-500 uppercase">Metrics</h4>
                                <button onClick={() => setShowAdvanced(!showAdvanced)} className="text-[10px] text-slate-400 hover:text-white flex items-center gap-1">
                                    {showAdvanced ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
                                    Details
                                </button>
                            </div>

                            <div className="grid grid-cols-3 gap-2">
                                <StatBox label="Travel" val={activeResult.score.geoDist} refVal={referenceResult?.score.geoDist} showDiff={props.showDiff} color="text-slate-200" suffix="" />
                                <StatBox label="Effort" val={activeResult.score.fingerUse} refVal={referenceResult?.score.fingerUse} showDiff={props.showDiff} color="text-slate-200" suffix="" />
                                <StatBox label="Imbal" val={activeResult.score.imbalancePenalty} refVal={referenceResult?.score.imbalancePenalty} showDiff={props.showDiff} color="text-slate-400" suffix="" />

                                <StatBox label="SFB" val={activeResult.score.statSfbBase + activeResult.score.statSfbLat + activeResult.score.statSfbLatWeak} total={activeResult.score.totalBigrams} showDiff={props.showDiff} color="text-red-400" />
                                <StatBox label="Scissor" val={activeResult.score.statScis} total={activeResult.score.totalBigrams} showDiff={props.showDiff} color="text-yellow-400" />
                                <StatBox label="Lat" val={activeResult.score.statLat} total={activeResult.score.totalBigrams} showDiff={props.showDiff} color="text-orange-400" />
                            </div>

                            {showAdvanced && (
                                <div className="grid grid-cols-3 gap-2 mt-2 pt-2 border-t border-slate-800">
                                    <StatBox label="Rolls" val={activeResult.score.statRoll} total={activeResult.score.totalBigrams} showDiff={props.showDiff} color="text-green-400" invertGood={true} />
                                    <StatBox label="Redir" val={activeResult.score.statRedir} total={activeResult.score.totalTrigrams} showDiff={props.showDiff} color="text-blue-400" />
                                    <StatBox label="Skips" val={activeResult.score.statSkip} total={activeResult.score.totalTrigrams} showDiff={props.showDiff} color="text-indigo-400" />
                                </div>
                            )}
                        </div>
                    </>
                )}

                {mode === 'optimize' && weights && searchParams && (
                    <>
                        <Card className="bg-slate-800/50 p-3">
                            <Label>Pinned Keys</Label>
                            <Input
                                value={props.pinnedKeys}
                                onChange={e => props.setPinnedKeys(e.target.value)}
                                placeholder="0:Q, 10:A..."
                                mono
                            />
                        </Card>

                        <OptimizerConfig
                            weights={weights}
                            searchParams={searchParams}
                            onWeightsChange={props.setWeights}
                            onParamsChange={props.setSearchParams}
                        />

                        <Card className="p-3 flex items-center justify-between">
                            <div>
                                <div className="text-xs font-bold text-slate-300">Local Worker</div>
                                <div className="text-[9px] text-slate-500">Donate CPU</div>
                            </div>
                            <input
                                type="checkbox"
                                checked={props.localWorkerEnabled}
                                onChange={e => props.toggleWorker(e.target.checked)}
                                className="accent-purple-500 h-4 w-4"
                            />
                        </Card>
                    </>
                )}
            </div>

            <div className="p-4 border-t border-slate-800 bg-slate-900/80">
                {mode === 'optimize' ? (
                    !activeJobId ? (
                        // FIXED: Using secondary variant for consistency
                        <Button variant="secondary" className="w-full" onClick={props.handleDispatch} icon={<Play size={16} />}>
                            START JOB
                        </Button>
                    ) : (
                        // Stop Job remains Danger for semantic reasons
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