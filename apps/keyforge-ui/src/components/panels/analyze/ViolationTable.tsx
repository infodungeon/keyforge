// apps/keyforge-ui/src/components/panels/analyze/ViolationTable.tsx

import { AlertTriangle } from "lucide-react";
import { MetricViolation } from "../../../types";

import { MapMode } from "../../KeyboardMap";

interface Props {
  title: string;
  items: MetricViolation[];
  color: string;
  mapMode: MapMode;
  totalScore: number;
}

export const ViolationTable = ({
  title,
  items,
  color,
  mapMode,
  totalScore,
}: Props) => {
  if (!items || items.length === 0) return null;
  return (
    <div className="mb-4">
      <h5
        className={`text-[10px] font-bold uppercase mb-2 ${color} flex items-center gap-1`}
      >
        <AlertTriangle size={10} /> {title}
      </h5>
      <div className="bg-slate-900/50 rounded border border-slate-800 text-[10px]">
        {items.slice(0, 5).map((v, i) => {
          const displayPct =
            mapMode === "penalty" ? (v.score / totalScore) * 100 : v.freq;
          return (
            <div
              key={i}
              className="flex justify-between p-1.5 border-b border-slate-800/50 last:border-0"
            >
              <span className="font-mono text-slate-300">{v.keys}</span>
              <div className="flex gap-3">
                <span className={`${color}`}>{displayPct.toFixed(2)}%</span>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};
