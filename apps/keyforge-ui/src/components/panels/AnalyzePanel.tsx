import { useState } from "react";
import { ValidationResult } from "../../types";
import { DerivedStats } from "../../utils";
import { SpaceHandPreference } from "../../services/stats";
import { ChevronDown, ChevronRight, ArrowRightLeft } from "lucide-react";
import { ButterflyChart } from "./analyze/ButterflyChart";
import { ViolationSection } from "./analyze/ViolationSection";
import { MetricGrid } from "./analyze/MetricGrid";

import { MapMode } from "../KeyboardMap";

interface Props {
  activeResult: ValidationResult | null;
  referenceResult: ValidationResult | null;
  derivedStats: DerivedStats | null;
  showDiff: boolean;
  setShowDiff: (b: boolean) => void;
  includeThumbs: boolean;
  setIncludeThumbs: (b: boolean) => void;
  spaceHand: SpaceHandPreference;
  setSpaceHand: (p: SpaceHandPreference) => void;
  mapMode?: MapMode;
}

export function AnalyzePanel({
  activeResult,
  referenceResult,
  derivedStats,
  showDiff,
  setShowDiff,
  includeThumbs,
  setIncludeThumbs,
  spaceHand,
  setSpaceHand,
  mapMode = "penalty",
}: Props) {
  const [showAdvanced, setShowAdvanced] = useState(false);

  if (!activeResult || !derivedStats) return null;

  return (
    <div className="space-y-6">
      <div className="bg-slate-800/50 rounded-xl p-4 border border-slate-700 relative overflow-hidden group">
        <div className="text-[10px] text-slate-500 uppercase tracking-widest font-bold mb-1">
          Total Score
        </div>
        <div className="text-4xl font-black text-transparent bg-clip-text bg-gradient-to-r from-blue-400 to-purple-400 font-mono">
          {activeResult.score.score.toFixed(0)}
        </div>
        {referenceResult && (
          <div className="mt-3 pt-3 border-t border-slate-700/50 flex items-center justify-between">
            <span className="text-[10px] text-slate-400">
              vs Ref:
              <span
                className={
                  referenceResult.score.score - activeResult.score.score > 0
                    ? "text-green-400 ml-1 font-bold"
                    : "text-red-400 ml-1 font-bold"
                }
              >
                {(
                  activeResult.score.score - referenceResult.score.score
                ).toFixed(0)}
              </span>
            </span>
            <button
              onClick={() => setShowDiff(!showDiff)}
              className={`text-[10px] flex items-center gap-1 px-2 py-1 rounded transition-colors ${
                showDiff
                  ? "bg-blue-500 text-white"
                  : "bg-slate-700/50 text-slate-400"
              }`}
            >
              <ArrowRightLeft size={10} /> {showDiff ? "Active" : "Compare"}
            </button>
          </div>
        )}
      </div>

      <div>
        <div className="flex items-center justify-between mb-3">
          <h4 className="text-[10px] font-bold text-slate-500 uppercase">
            Balance
          </h4>
          <div className="flex flex-col gap-2 items-end">
            <div className="flex items-center gap-2">
              <input
                type="checkbox"
                id="include-thumbs"
                checked={includeThumbs}
                onChange={(e) => setIncludeThumbs(e.target.checked)}
                className="accent-blue-500 h-3 w-3 rounded border-slate-700 bg-slate-900"
              />
              <label
                htmlFor="include-thumbs"
                className="text-[9px] text-slate-400 cursor-pointer select-none"
              >
                Include Thumbs
              </label>
            </div>
            {includeThumbs && (
              <div className="flex flex-col gap-1 items-end">
                <span className="text-[8px] text-slate-500 uppercase font-bold">
                  Space Hand Preference
                </span>
                <div className="flex bg-slate-800 rounded p-0.5 border border-slate-700">
                  {(["left", "bilateral", "right"] as const).map((opt) => (
                    <button
                      key={opt}
                      onClick={() => setSpaceHand(opt)}
                      className={`px-2 py-0.5 text-[8px] uppercase font-bold rounded transition-colors ${
                        spaceHand === opt
                          ? "bg-blue-600 text-white"
                          : "text-slate-500 hover:text-slate-300"
                      }`}
                    >
                      {opt === "bilateral" ? "Both" : opt}
                    </button>
                  ))}
                </div>
              </div>
            )}
          </div>
        </div>
        <div className="flex gap-1 h-3 rounded-full overflow-hidden mb-1">
          <div
            className="bg-blue-500 transition-all"
            style={{ width: `${derivedStats.handBalance.left}%` }}
          />
          <div
            className="bg-purple-500 transition-all"
            style={{ width: `${derivedStats.handBalance.right}%` }}
          />
        </div>
        <div className="flex justify-between text-[10px] text-slate-300 mb-4 font-mono font-bold">
          <span>L: {derivedStats.handBalance.left.toFixed(1)}%</span>
          <span>R: {derivedStats.handBalance.right.toFixed(1)}%</span>
        </div>
      </div>

      <div>
        <h4 className="text-[10px] font-bold text-slate-500 uppercase mb-3">
          Finger Load
        </h4>
        <ButterflyChart
          left={derivedStats.fingerUsage.left}
          right={derivedStats.fingerUsage.right}
        />
      </div>

      {/* Metrics Grid */}
      <div>
        <div className="flex items-center justify-between mb-3">
          <h4 className="text-[10px] font-bold text-slate-500 uppercase">
            Metrics
          </h4>
          <button
            onClick={() => setShowAdvanced(!showAdvanced)}
            className="text-[10px] text-slate-400 hover:text-white flex items-center gap-1"
          >
            {showAdvanced ? (
              <ChevronDown size={12} />
            ) : (
              <ChevronRight size={12} />
            )}
            Details
          </button>
        </div>

        <MetricGrid
          activeResult={activeResult}
          referenceResult={referenceResult}
          mapMode={mapMode}
          includeThumbs={includeThumbs}
          derivedStats={derivedStats}
          showDiff={showDiff}
        />

        {showAdvanced && (
          <ViolationSection activeResult={activeResult} mapMode={mapMode} />
        )}
      </div>
    </div>
  );
}
