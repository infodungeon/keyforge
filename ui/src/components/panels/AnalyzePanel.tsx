// ===== keyforge/ui/src/components/panels/AnalyzePanel.tsx =====
import { useState } from "react";
import { ValidationResult, MetricViolation } from "../../types";
import { DerivedStats } from "../../utils";
import { StatBox, FingerBar } from "../Charts";
import { ChevronDown, ChevronRight, ArrowRightLeft, AlertTriangle } from "lucide-react";

interface Props {
    activeResult: ValidationResult | null;
    referenceResult: ValidationResult | null;
    derivedStats: DerivedStats | null;
    showDiff: boolean;
    setShowDiff: (b: boolean) => void;
}

const ViolationTable = ({ title, items, color }: { title: string, items: MetricViolation[], color: string }) => {
    if (!items || items.length === 0) return null;
    return (
        <div className="mb-4">
            <h5 className={`text-[10px] font-bold uppercase mb-2 ${color} flex items-center gap-1`}>
                <AlertTriangle size={10} /> {title}
            </h5>
            <div className="bg-slate-900/50 rounded border border-slate-800 text-[10px]">
                {items.slice(0, 5).map((v, i) => (
                    <div key={i} className="flex justify-between p-1.5 border-b border-slate-800/50 last:border-0">
                        <span className="font-mono text-slate-300">{v.keys}</span>
                        <div className="flex gap-3">
                            <span className="text-slate-500">{v.freq.toFixed(0)}</span>
                            <span className={`${color.replace("text-", "text-")}`}>{v.score.toFixed(0)}</span>
                        </div>
                    </div>
                ))}
            </div>
        </div>
    );
};

export function AnalyzePanel({ activeResult, referenceResult, derivedStats, showDiff, setShowDiff }: Props) {
    const [showAdvanced, setShowAdvanced] = useState(false);

    if (!activeResult || !derivedStats) return null;

    return (
        <div className="space-y-6">
            {/* ... (Previous Score/Balance/Finger components unchanged) ... */}
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
                        <button onClick={() => setShowDiff(!showDiff)}
                            className={`text-[10px] flex items-center gap-1 px-2 py-1 rounded transition-colors ${showDiff ? "bg-blue-500 text-white" : "bg-slate-700/50 text-slate-400"}`}>
                            <ArrowRightLeft size={10} /> {showDiff ? "Active" : "Compare"}
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

            {/* Metrics Grid */}
            <div>
                <div className="flex items-center justify-between mb-3">
                    <h4 className="text-[10px] font-bold text-slate-500 uppercase">Metrics</h4>
                    <button onClick={() => setShowAdvanced(!showAdvanced)} className="text-[10px] text-slate-400 hover:text-white flex items-center gap-1">
                        {showAdvanced ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
                        Details
                    </button>
                </div>

                <div className="grid grid-cols-3 gap-2">
                    <StatBox label="Travel" val={activeResult.score.geoDist} refVal={referenceResult?.score.geoDist} showDiff={showDiff} color="text-slate-200" suffix="" />
                    <StatBox label="Effort" val={activeResult.score.fingerUse} refVal={referenceResult?.score.fingerUse} showDiff={showDiff} color="text-slate-200" suffix="" />
                    <StatBox label="Imbal" val={activeResult.score.imbalancePenalty} refVal={referenceResult?.score.imbalancePenalty} showDiff={showDiff} color="text-slate-400" suffix="" />

                    <StatBox label="SFB" val={activeResult.score.statSfbBase + activeResult.score.statSfbLat + activeResult.score.statSfbLatWeak} total={activeResult.score.totalBigrams} showDiff={showDiff} color="text-red-400" />
                    <StatBox label="Scissor" val={activeResult.score.statScis} total={activeResult.score.totalBigrams} showDiff={showDiff} color="text-yellow-400" />
                    <StatBox label="Lat" val={activeResult.score.statLat} total={activeResult.score.totalBigrams} showDiff={showDiff} color="text-orange-400" />
                </div>

                {showAdvanced && (
                    <div className="mt-4 pt-4 border-t border-slate-800 animate-in fade-in slide-in-from-top-2">
                        {/* New Violation Tables */}
                        <ViolationTable title="Top SFBs" items={activeResult.score.topSfbs} color="text-red-400" />
                        <ViolationTable title="Top Scissors" items={activeResult.score.topScissors} color="text-yellow-400" />
                        <ViolationTable title="Top Redirects" items={activeResult.score.topRedirs} color="text-blue-400" />

                        <div className="grid grid-cols-3 gap-2 mt-2 pt-2 border-t border-slate-800">
                            <StatBox label="Rolls" val={activeResult.score.statRoll} total={activeResult.score.totalBigrams} showDiff={showDiff} color="text-green-400" invertGood={true} />
                            <StatBox label="Redir" val={activeResult.score.statRedir} total={activeResult.score.totalTrigrams} showDiff={showDiff} color="text-blue-400" />
                            <StatBox label="Skips" val={activeResult.score.statSkip} total={activeResult.score.totalTrigrams} showDiff={showDiff} color="text-indigo-400" />
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
}