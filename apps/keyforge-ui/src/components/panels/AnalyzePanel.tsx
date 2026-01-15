import { useState } from "react";
import { ValidationResult, MetricViolation } from "../../types";
import { DerivedStats } from "../../utils";
import { StatBox } from "../Charts";
import { SpaceHandPreference } from "../../services/stats";
import {
  ChevronDown,
  ChevronRight,
  ArrowRightLeft,
  AlertTriangle,
} from "lucide-react";

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
}

const ViolationTable = ({
  title,
  items,
  color,
}: {
  title: string;
  items: MetricViolation[];
  color: string;
}) => {
  if (!items || items.length === 0) return null;
  return (
    <div className="mb-4">
      <h5
        className={`text-[10px] font-bold uppercase mb-2 ${color} flex items-center gap-1`}
      >
        <AlertTriangle size={10} /> {title}
      </h5>
      <div className="bg-slate-900/50 rounded border border-slate-800 text-[10px]">
        {items.slice(0, 5).map((v, i) => (
          <div
            key={i}
            className="flex justify-between p-1.5 border-b border-slate-800/50 last:border-0"
          >
            <span className="font-mono text-slate-300">{v.keys}</span>
            <div className="flex gap-3">
              <span className="text-slate-500">{v.freq.toFixed(0)}</span>
              <span className={`${color.replace("text-", "text-")}`}>
                {v.score.toFixed(0)}
              </span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

const ButterflyChart = ({
  left,
  right,
}: {
  left: number[];
  right: number[];
}) => {
  // Finger indices: 0=Thumb, 1=Index, 2=Mid, 3=Ring, 4=Pinky
  // Labels for the rows
  const labels = ["Pinky", "Ring", "Mid", "Index", "Thumb"];

  // We map the display rows (0..4) to the actual finger indices
  // Top row (0) = Pinky (index 4)
  // Bottom row (4) = Thumb (index 0)
  const mapRowToFingerIdx = (row: number) => 4 - row;

  const Row = ({ rowIdx }: { rowIdx: number }) => {
    const fingerIdx = mapRowToFingerIdx(rowIdx);
    const lVal = left[fingerIdx] || 0;
    const rVal = right[fingerIdx] || 0;
    const label = labels[rowIdx];

    // Colors per finger index (0..4)
    const colors = [
      "bg-slate-500", // Thumb
      "bg-green-500", // Index
      "bg-blue-500", // Mid
      "bg-purple-500", // Ring
      "bg-pink-500", // Pinky
    ];
    const color = colors[fingerIdx];

    return (
      <div className="flex items-center gap-2 text-[9px] mb-1.5">
        {/* Left Side (Right Aligned) */}
        <div className="flex-1 flex justify-end items-center gap-2">
          <span className="text-slate-500 font-mono w-8 text-right">
            {lVal.toFixed(1)}%
          </span>
          <div className="h-1.5 w-24 bg-slate-800/50 rounded-l-full overflow-hidden flex justify-end">
            <div
              className={`h-full ${color} opacity-80`}
              style={{ width: `${Math.min(100, lVal * 4)}%` }}
            />
          </div>
        </div>

        {/* Center Label */}
        <div className="w-10 text-center text-slate-600 font-bold uppercase text-[8px]">
          {label}
        </div>

        {/* Right Side (Left Aligned) */}
        <div className="flex-1 flex justify-start items-center gap-2">
          <div className="h-1.5 w-24 bg-slate-800/50 rounded-r-full overflow-hidden flex justify-start">
            <div
              className={`h-full ${color} opacity-80`}
              style={{ width: `${Math.min(100, rVal * 4)}%` }}
            />
          </div>
          <span className="text-slate-500 font-mono w-8 text-left">
            {rVal.toFixed(1)}%
          </span>
        </div>
      </div>
    );
  };

  return (
    <div className="flex flex-col w-full">
      {[0, 1, 2, 3, 4].map((i) => (
        <Row key={i} rowIdx={i} />
      ))}
    </div>
  );
};

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
              className={`text-[10px] flex items-center gap-1 px-2 py-1 rounded transition-colors ${showDiff ? "bg-blue-500 text-white" : "bg-slate-700/50 text-slate-400"}`}
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
              <label htmlFor="include-thumbs" className="text-[9px] text-slate-400 cursor-pointer select-none">
                Include Thumbs
              </label>
            </div>
            {includeThumbs && (
              <div className="flex flex-col gap-1 items-end">
                <span className="text-[8px] text-slate-500 uppercase font-bold">Space Hand Preference</span>
                <div className="flex bg-slate-800 rounded p-0.5 border border-slate-700">
                  {(["left", "bilateral", "right"] as const).map((opt) => (
                    <button
                      key={opt}
                      onClick={() => setSpaceHand(opt)}
                      className={`px-2 py-0.5 text-[8px] uppercase font-bold rounded transition-colors ${
                        spaceHand === opt ? "bg-blue-600 text-white" : "text-slate-500 hover:text-slate-300"
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

        <div className="grid grid-cols-3 gap-2">
          <StatBox
            label="Travel/Key"
            val={(activeResult.score.distance / 100000) * 100}
            refVal={referenceResult ? (referenceResult.score.distance / 100000) * 100 : undefined}
            showDiff={showDiff}
            color="text-slate-200"
            suffix="%"
            precision={2}
          />
          <StatBox
            label="Imbal"
            val={activeResult.score.hand_balance}
            refVal={referenceResult?.score.hand_balance}
            showDiff={showDiff}
            color="text-slate-400"
            suffix=""
          />

          <StatBox
            label="SFB"
            val={activeResult.score.sfb_total}
            showDiff={showDiff}
            color="text-red-400"
            suffix="%"
            precision={2}
          />
          <StatBox
            label="Scissor"
            val={activeResult.score.scissors}
            showDiff={showDiff}
            color="text-yellow-400"
            suffix="%"
            precision={2}
          />
          <StatBox
            label="Redir"
            val={activeResult.score.redirects}
            showDiff={showDiff}
            color="text-blue-400"
            suffix="%"
            precision={2}
          />
          <StatBox
            label="Rolls"
            val={activeResult.score.rolls}
            showDiff={showDiff}
            color="text-green-400"
            suffix="%"
            precision={2}
            invertGood={true}
          />
        </div>

        {showAdvanced && (
          <div className="mt-4 pt-4 border-t border-slate-800 animate-in fade-in slide-in-from-top-2">
            {/* New Violation Tables */}
            <ViolationTable
              title="Top SFBs"
              items={activeResult.score.top_sfbs}
              color="text-red-400"
            />
            <ViolationTable
              title="Top Scissors"
              items={activeResult.score.top_scissors}
              color="text-yellow-400"
            />
            <ViolationTable
              title="Top Redirects"
              items={activeResult.score.top_redirs}
              color="text-blue-400"
            />
          </div>
        )}
      </div>
    </div>
  );
}
