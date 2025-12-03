interface StatProps {
  label: string;
  val?: number;
  refVal?: number;
  total?: number;
  color: string;
  suffix?: string;
  showDiff?: boolean;
  invertGood?: boolean;
}

export const StatBox = ({ label, val, refVal, total, color, suffix, showDiff, invertGood }: StatProps) => {
  let display = "--";
  let activeColor = color;

  // Imbalance Special Logic: 0 is "Perfect"
  const isImbalance = label.includes("Imbalance");

  if (val !== undefined) {
    if (showDiff && refVal !== undefined) {
      const diffAbs = val - refVal;
      const diffPct = total ? ((diffAbs / total) * 100) : diffAbs;
      const isBetter = invertGood ? diffAbs > 0 : diffAbs < 0;
      activeColor = isBetter ? "text-green-400" : "text-red-400";
      const sign = diffAbs > 0 ? "+" : "";
      display = `${sign}${total ? diffPct.toFixed(2) + "%" : diffAbs.toFixed(0)}`;
    } else {
      if (isImbalance) {
        if (val === 0) { display = "Perfect"; activeColor = "text-green-400"; }
        else { display = val.toFixed(0) + (suffix || ""); activeColor = "text-orange-400"; }
      } else if (total) {
        display = ((val / total) * 100).toFixed(2) + "%";
      } else {
        display = val.toFixed(0) + (suffix || "");
      }
    }
  }

  return (
    <div className="bg-slate-800/30 p-2.5 rounded border border-slate-800 hover:border-slate-700 transition-colors">
      <div className="text-[9px] text-slate-500 uppercase tracking-tight flex justify-between">
        {label}
        {showDiff && <span className="text-[8px] opacity-50">Δ</span>}
      </div>
      <div className={`text-sm font-bold ${activeColor} font-mono tracking-tight`}>{display}</div>
    </div>
  );
};

export const FingerBar = ({ label, pct, color }: { label: string, pct: number, color: string }) => {
  const validPct = isNaN(pct) ? 0 : pct;
  return (
    <div className="flex items-center gap-2 text-[10px]">
      <div className="w-8 text-slate-400 text-right">{label}</div>
      <div className="flex-1 h-1.5 bg-slate-800 rounded-full overflow-hidden">
        <div className={`h-full ${color} rounded-full transition-all duration-500`} style={{ width: `${validPct}%` }} />
      </div>
      <div className="w-8 text-slate-500 font-mono text-right">{validPct.toFixed(1)}%</div>
    </div>
  );
};