// ===== keyforge/ui/src/components/panels/ArenaPanel.tsx =====
import { useState } from "react";
import { Card } from "../ui/Card";
import { Button } from "../ui/Button";
import { Label } from "../ui/Label";
import { 
    RotateCcw, Trash2, UserCheck, Lock, Unlock, 
    Minus, Plus, ChevronDown, ChevronRight 
} from "lucide-react";
import { useKeyboard } from "../../context/KeyboardContext";
import { useArena, ZOOM_LEVELS } from "../../context/ArenaContext";

export function ArenaPanel() {
    const { 
        wpm, accuracy, coveragePct, sampleCount,
        zoomIndex, stopOnError, isGenerating,
        nextSession, resetData, generateProfile,
        setStopOnError, changeZoom
    } = useArena();

    const { weights } = useKeyboard();
    const [showWeights, setShowWeights] = useState(false);

    return (
        <div className="space-y-6">
            
            {/* 1. Progress Stats */}
            <Card className="bg-slate-800/50 p-4 space-y-4">
                <div className="grid grid-cols-2 gap-4 text-center">
                    <div>
                        <div className="text-[10px] text-slate-500 uppercase font-bold">Speed</div>
                        <div className="text-2xl font-black text-purple-400">{wpm} <span className="text-xs text-slate-600">WPM</span></div>
                    </div>
                    <div>
                        <div className="text-[10px] text-slate-500 uppercase font-bold">Accuracy</div>
                        <div className="text-2xl font-black text-blue-400">{accuracy}<span className="text-sm">%</span></div>
                    </div>
                </div>

                <div>
                    <div className="flex justify-between text-[10px] font-bold text-slate-500 uppercase mb-1.5">
                        <span>Biometric Saturation</span>
                        <span>{coveragePct.toFixed(1)}%</span>
                    </div>
                    <div className="w-full h-2 bg-slate-900 rounded-full overflow-hidden border border-slate-800">
                        <div
                            className="h-full bg-gradient-to-r from-blue-600 to-green-400 transition-all duration-500"
                            style={{ width: `${Math.min(100, coveragePct)}%` }}
                        />
                    </div>
                    <div className="text-[9px] text-slate-600 text-right mt-1">
                        {sampleCount} data points collected
                    </div>
                </div>
            </Card>

            {/* 2. Controls */}
            <div className="space-y-4">
                <div className="flex flex-col gap-2">
                    <Label>Controls</Label>
                    <div className="grid grid-cols-2 gap-2">
                        <Button variant="secondary" onClick={nextSession} icon={<RotateCcw size={14} />}>
                            Restart
                        </Button>
                        <button 
                            onClick={() => setStopOnError(!stopOnError)}
                            className={`flex items-center justify-center gap-2 px-3 py-2 rounded-lg border text-[10px] font-bold uppercase transition-colors ${stopOnError ? "bg-red-900/20 border-red-800 text-red-400" : "bg-slate-800 border-slate-700 text-slate-400 hover:bg-slate-700"}`}
                        >
                            {stopOnError ? <Lock size={14} /> : <Unlock size={14} />}
                            {stopOnError ? "Locked" : "Free"}
                        </button>
                    </div>
                </div>

                <div>
                    <Label>Font Size</Label>
                    <div className="flex items-center bg-slate-950 border border-slate-800 rounded-lg p-1">
                        <button onClick={() => changeZoom(-1)} className="p-2 hover:bg-slate-800 text-slate-500 hover:text-white rounded transition-colors"><Minus size={14} /></button>
                        <span className="flex-1 text-center text-xs font-mono font-bold text-slate-400">
                            {Math.round((zoomIndex / (ZOOM_LEVELS.length - 1)) * 100)}%
                        </span>
                        <button onClick={() => changeZoom(1)} className="p-2 hover:bg-slate-800 text-slate-500 hover:text-white rounded transition-colors"><Plus size={14} /></button>
                    </div>
                </div>
            </div>

            {/* 3. Data Management */}
            <Card className="bg-slate-900 border-slate-800 p-4 space-y-3">
                <Label>Data Management</Label>
                <Button
                    variant="primary"
                    className="w-full"
                    onClick={generateProfile}
                    isLoading={isGenerating}
                    icon={<UserCheck size={14} />}
                    disabled={coveragePct < 5} 
                    title={coveragePct < 5 ? "Need more data (~5%)" : "Generate CSV"}
                >
                    GENERATE PROFILE
                </Button>
                
                <Button
                    variant="danger"
                    className="w-full opacity-80 hover:opacity-100"
                    onClick={() => {
                        if (confirm("Reset all biometric progress? This cannot be undone.")) resetData();
                    }}
                    icon={<Trash2 size={14} />}
                >
                    RESET DATA
                </Button>
            </Card>

            {/* 4. Scoring Weights (Reference) */}
            {weights && (
                <div className="border-t border-slate-800 pt-4">
                    <button 
                        onClick={() => setShowWeights(!showWeights)}
                        className="flex items-center justify-between w-full text-[10px] font-bold text-slate-500 uppercase hover:text-slate-300 transition-colors"
                    >
                        <span>Physics Configuration</span>
                        {showWeights ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
                    </button>
                    
                    {showWeights && (
                        <div className="mt-3 space-y-1 max-h-40 overflow-y-auto custom-scrollbar pr-2">
                            {Object.entries(weights).map(([k, v]) => (
                                <div key={k} className="flex justify-between text-[10px] border-b border-slate-800/50 pb-1 last:border-0">
                                    <span className="text-slate-400 truncate w-32" title={k}>
                                        {k.replace('penalty_', '').replace('bonus_', '').replace(/_/g, ' ')}
                                    </span>
                                    <span className="font-mono text-slate-200">{v}</span>
                                </div>
                            ))}
                        </div>
                    )}
                </div>
            )}
        </div>
    );
}