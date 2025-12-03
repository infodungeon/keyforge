import { ValidationResult } from "../types";

interface Props {
  history: ValidationResult[];
  onSelect: (res: ValidationResult) => void;
  isSearching: boolean;
}

const pct = (val: number, total: number) => total ? ((val / total) * 100).toFixed(2) : "0.00";

export function HistoryTable({ history, onSelect, isSearching }: Props) {
  return (
    <div className="h-64 bg-slate-900 border-t border-slate-800 overflow-auto">
      <table className="w-full text-xs text-left border-collapse">
        <thead className="bg-slate-950 text-slate-400 font-bold sticky top-0 z-10">
          <tr>
            <th className="p-2 border-b border-slate-800 w-32">Layout</th>
            <th className="p-2 border-b border-slate-800 text-right text-white">Score</th>
            <th className="p-2 border-b border-slate-800 text-right text-red-400">SFB%</th>
            <th className="p-2 border-b border-slate-800 text-right text-red-400">Lat%</th>
            <th className="p-2 border-b border-slate-800 text-right text-yellow-400">Scis%</th>
            <th className="p-2 border-b border-slate-800 text-right text-green-400">Roll%</th>
            <th className="p-2 border-b border-slate-800 text-right text-blue-400">Redir%</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-slate-800">
          {history.map((res, idx) => {
            const s = res.score;
            const tBi = s.totalBigrams || 1;
            const tTri = s.totalTrigrams || 1;
            return (
              <tr
                key={idx}
                onClick={() => !isSearching && onSelect(res)}
                className={`cursor-pointer hover:bg-slate-800 ${isSearching ? 'opacity-50 pointer-events-none' : ''}`}
              >
                <td className="p-2 font-bold text-white">{res.layoutName}</td>
                <td className="p-2 text-right font-mono text-blue-400">{(s.layoutScore).toFixed(0)}</td>
                <td className="p-2 text-right text-slate-300">{pct(s.statSfbBase, tBi)}%</td>
                <td className="p-2 text-right text-slate-300">{pct(s.statSfbLat, tBi)}%</td>
                <td className="p-2 text-right text-yellow-300">{pct(s.statScis, tBi)}%</td>
                <td className="p-2 text-right text-green-400">{pct(s.statRoll, tBi)}%</td>
                <td className="p-2 text-right text-blue-400">{pct(s.statRedir, tTri)}%</td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}