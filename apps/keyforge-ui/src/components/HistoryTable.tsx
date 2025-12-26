import { ValidationResult } from "../types";

interface Props {
  history: ValidationResult[];
  onSelect: (res: ValidationResult) => void;
  isSearching: boolean;
}

const pct = (val: number) => (val * 100).toFixed(2);

export function HistoryTable({ history, onSelect, isSearching }: Props) {
  return (
    <div className="h-64 bg-slate-900 border-t border-slate-800 overflow-auto">
      <table className="w-full text-xs text-left border-collapse">
        <thead className="bg-slate-950 text-slate-400 font-bold sticky top-0 z-10">
          <tr>
            <th className="p-2 border-b border-slate-800 w-32">Layout</th>
            <th className="p-2 border-b border-slate-800 text-right text-white">
              Score
            </th>
            <th className="p-2 border-b border-slate-800 text-right text-red-400">
              SFB%
            </th>
            <th className="p-2 border-b border-slate-800 text-right text-yellow-400">
              Scis
            </th>
            <th className="p-2 border-b border-slate-800 text-right text-green-400">
              Rolls
            </th>
            <th className="p-2 border-b border-slate-800 text-right text-blue-400">
              Redir
            </th>
          </tr>
        </thead>
        <tbody className="divide-y divide-slate-800">
          {history.map((res, idx) => {
            const s = res.score;
            return (
              <tr
                key={idx}
                onClick={() => !isSearching && onSelect(res)}
                className={`cursor-pointer hover:bg-slate-800 ${isSearching ? "opacity-50 pointer-events-none" : ""}`}
              >
                <td className="p-2 font-bold text-white">{res.layout_name}</td>
                <td className="p-2 text-right font-mono text-blue-400">
                  {s.score.toFixed(0)}
                </td>
                <td className="p-2 text-right text-slate-300">
                  {pct(s.sfb_ratio)}%
                </td>
                <td className="p-2 text-right text-yellow-300">
                  {s.scissors.toFixed(0)}
                </td>
                <td className="p-2 text-right text-green-400">
                  {s.rolls.toFixed(0)}
                </td>
                <td className="p-2 text-right text-blue-400">
                  {s.redirects.toFixed(0)}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
