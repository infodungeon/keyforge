import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ValidationResult } from "./types";
import { KeyboardMap } from "./components/KeyboardMap";
import "./App.css";

// Standard Layout Presets
const PRESETS: Record<string, string> = {
  "Canary": "WLYPBZFOU'CRSTGMNEIAQJVDKXH,./",
  "Colemak": "QWFPGJLUY;ARSTDHNEIOZXCVBKM,./",
  "Colemak-DH": "QWFPBJLUY;ARSTGMNEIOZXCDVKH,./",
  "Dvorak": "',.PYFGCRLAOEUIDHTNS;QJKXBMWVZ",
  "Engram": "BYOU'LDWVZCIEA,HTSNQGXJKMFP;./",
  "Gallium": "BLDCVJYOU,NRTSGPHAEIXQMWZKF';.",
  "Graphite": "BLDWZ'FOUJNRTSGYHAEIQXMCVKP,./",
  "Hands Down Ref": "XRYBPJLCU;SNHTGMOEAIZWVDKQF,./",
  "Qwerty": "QWERTYUIOPASDFGHJKL;ZXCVBNM,./",
  "Sturdy": "VMLCPXFOUJSTRYDNAEIHZKQGWB';.,",
  "Workman": "QDRWBJFUP;ASHTGYNEOIZXMCVKL,./",
};

// 1. Sort Layouts Alphabetically for Dropdown
const SORTED_PRESETS = Object.keys(PRESETS).sort();

function App() {
  const [isLoaded, setIsLoaded] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 2. Default to QWERTY
  const [layoutName, setLayoutName] = useState("Qwerty");
  const [layoutString, setLayoutString] = useState(PRESETS["Qwerty"]);

  const [activeResult, setActiveResult] = useState<ValidationResult | null>(null);
  const [history, setHistory] = useState<ValidationResult[]>([]);

  // Config path (Adjust default if needed)
  const [projectPath, setProjectPath] = useState("/home/robert/Documents/KeyboardLayouts/Data Driven Analysis/keyforge");

  // 1. Auto-Load Engine
  useEffect(() => {
    async function init() {
      try {
        await invoke("cmd_load_dataset", { basePath: projectPath });
        setIsLoaded(true);
        // Auto-validate Qwerty on load so the screen isn't empty
        await validateInternal("Qwerty", PRESETS["Qwerty"]);
      } catch (e) {
        setError(`Failed to load engine: ${e}`);
      }
    }
    init();
  }, []);

  // 2. Validation Logic
  const validateInternal = async (name: string, str: string) => {
    if (str.length < 30) return;
    try {
      // Invoke Rust command
      const res = await invoke<ValidationResult>("cmd_validate_layout", {
        layoutStr: str,
        weights: null
      });

      // Ensure the result has the name we expect (Rust might return "Custom")
      const finalRes = { ...res, layoutName: name };

      // Update History: Remove existing entry with same name, add new one to top
      setHistory(prev => {
        const filtered = prev.filter(x => x.layoutName !== name);
        return [finalRes, ...filtered];
      });

      // Set Active
      setActiveResult(finalRes);
    } catch (e) {
      console.error(e);
      alert(`Validation failed: ${e}`);
    }
  };

  const handleAddLayout = () => validateInternal(layoutName || "Custom", layoutString);

  const handlePresetChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const name = e.target.value;
    if (PRESETS[name]) {
      setLayoutName(name);
      setLayoutString(PRESETS[name]);
      // Auto-validate on selection for better UX
      if (isLoaded) validateInternal(name, PRESETS[name]);
    }
  };

  // Percentage Helper
  const pct = (val: number, total: number) => {
    if (!total || total === 0) return "0.00";
    return ((val / total) * 100).toFixed(2);
  };

  if (error) return <div className="p-8 text-red-500 bg-red-100 h-screen flex items-center justify-center font-mono">{error}</div>;
  if (!isLoaded) return <div className="p-8 text-blue-500 h-screen flex items-center justify-center animate-pulse font-bold text-xl">Loading KeyForge Engine...</div>;

  return (
    <div className="min-h-screen bg-slate-950 text-slate-200 p-6 font-sans flex flex-col gap-6">

      {/* --- HEADER --- */}
      <header className="flex flex-wrap items-center gap-4 border-b border-slate-800 pb-4">
        <h1 className="text-3xl font-black tracking-tight text-white">
          KEY<span className="text-blue-500">FORGE</span>
        </h1>
        <div className="ml-auto flex gap-2 items-center">
          <label className="text-xs text-slate-500 font-mono font-bold">REPO PATH:</label>
          <input
            className="bg-slate-900 border border-slate-800 text-xs p-2 rounded text-slate-400 w-96 font-mono focus:border-blue-500 outline-none"
            value={projectPath}
            onChange={(e) => setProjectPath(e.target.value)}
          />
        </div>
      </header>

      {/* --- CONTROL BAR --- */}
      <div className="bg-slate-900 p-4 rounded-xl border border-slate-800 shadow-lg flex flex-wrap gap-4 items-end">

        {/* Layout Selector */}
        <div className="flex flex-col gap-1 w-48">
          <label className="text-[10px] font-bold text-slate-500 uppercase tracking-wider">Select Layout</label>
          <select
            className="h-10 bg-slate-800 border border-slate-700 rounded-lg px-3 text-sm focus:ring-2 focus:ring-blue-600 outline-none text-white cursor-pointer"
            value={PRESETS[layoutName] === layoutString ? layoutName : "Custom"}
            onChange={handlePresetChange}
          >
            <option value="Custom">Custom / Edited</option>
            {SORTED_PRESETS.map(k => <option key={k} value={k}>{k}</option>)}
          </select>
        </div>

        {/* Name Input */}
        <div className="flex flex-col gap-1 w-40">
          <label className="text-[10px] font-bold text-slate-500 uppercase tracking-wider">Name</label>
          <input
            className="h-10 bg-slate-800 border border-slate-700 rounded-lg px-3 text-sm focus:ring-2 focus:ring-blue-600 outline-none text-white placeholder-slate-600"
            value={layoutName}
            onChange={(e) => setLayoutName(e.target.value)}
            placeholder="Custom Name"
          />
        </div>

        {/* Wide Layout Input */}
        <div className="flex flex-col gap-1 flex-1 min-w-[400px]">
          <label className="text-[10px] font-bold text-slate-500 uppercase tracking-wider flex justify-between">
            <span>Mapping String</span>
            <span className={layoutString.length === 30 ? "text-green-500" : "text-red-500"}>{layoutString.length}/30</span>
          </label>
          <input
            className="h-10 w-full font-mono text-lg bg-slate-800 border border-slate-700 rounded-lg px-3 focus:ring-2 focus:ring-blue-600 outline-none uppercase tracking-widest text-white placeholder-slate-600"
            value={layoutString}
            maxLength={30}
            onChange={(e) => setLayoutString(e.target.value.toUpperCase())}
            placeholder="QWERTY..."
          />
        </div>

        <button
          onClick={handleAddLayout}
          disabled={layoutString.length !== 30}
          className="h-10 px-6 bg-blue-600 hover:bg-blue-500 disabled:bg-slate-800 disabled:text-slate-600 text-white font-bold rounded-lg transition shadow-md whitespace-nowrap"
        >
          ANALYZE
        </button>
      </div>

      {/* --- VISUALIZER (Centered) --- */}
      {activeResult && (
        <div className="flex flex-col items-center justify-center gap-2">
          <div className="bg-slate-900 border border-slate-800 rounded-2xl p-6 shadow-2xl w-fit">
            <h2 className="text-center text-xl font-black text-white mb-4 tracking-wide uppercase">
              {activeResult.layoutName}
            </h2>
            {/* Pass the calculated heatmap from Rust */}
            <KeyboardMap
              geometry={activeResult.geometry}
              layoutString={layoutString}
              heatmap={activeResult.heatmap}
            />
          </div>
        </div>
      )}

      {/* --- COMPARISON TABLE --- */}
      {history.length > 0 && (
        <div className="overflow-x-auto rounded-xl border border-slate-800 shadow-xl bg-slate-900">
          <table className="w-full text-sm text-left border-collapse">
            <thead className="bg-slate-950 text-slate-400 uppercase font-bold text-[10px] tracking-wider">
              <tr>
                <th className="p-3 border-b border-slate-800 text-left w-32 sticky left-0 bg-slate-950 z-10 border-r border-slate-800">Layout</th>
                <th className="p-3 border-b border-slate-800 text-right text-white w-28 bg-slate-900/50">Score</th>

                {/* SFB Group */}
                <th className="p-3 border-b border-slate-800 text-right text-red-400 w-20 border-l border-slate-800">Base%</th>
                <th className="p-3 border-b border-slate-800 text-right text-red-400 w-20">Lat%</th>
                <th className="p-3 border-b border-slate-800 text-right text-red-400 w-20">WkL%</th>
                <th className="p-3 border-b border-slate-800 text-right text-red-400 w-20">Diag%</th>
                <th className="p-3 border-b border-slate-800 text-right text-red-400 w-20">Long%</th>
                <th className="p-3 border-b border-slate-800 text-right text-red-400 w-20">Bot%</th>

                {/* Mechanics Group */}
                <th className="p-3 border-b border-slate-800 text-right text-orange-400 w-20 border-l border-slate-800">LSB%</th>
                <th className="p-3 border-b border-slate-800 text-right text-yellow-400 w-20">Scis%</th>
                <th className="p-3 border-b border-slate-800 text-right text-purple-400 w-20">Pinky%</th>

                {/* Flow Group */}
                <th className="p-3 border-b border-slate-800 text-right text-green-400 w-20 border-l border-slate-800">Rol2%</th>
                <th className="p-3 border-b border-slate-800 text-right text-green-400 w-20">Rol3%</th>
                <th className="p-3 border-b border-slate-800 text-right text-blue-400 w-20">Redir%</th>

                {/* Cost Raw */}
                <th className="p-3 border-b border-slate-800 text-right text-slate-500 w-24 border-l border-slate-800">Flow Cost</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-800">
              {history.map((res, idx) => {
                const s = res.score;
                // Use CamelCase field names from JSON
                const tBi = s.totalBigrams || 1;
                const tTri = s.totalTrigrams || 1;
                const tChar = s.totalChars || 1;

                const isActive = activeResult?.layoutName === res.layoutName;

                return (
                  <tr
                    key={idx}
                    onClick={() => {
                      setActiveResult(res);
                      setLayoutName(res.layoutName);
                      // Note: We don't persist the layoutString in history currently,
                      // so clicking a row won't update the input box string, just the visualizer/stats.
                    }}
                    className={`cursor-pointer transition hover:bg-slate-800 ${isActive ? "bg-slate-800/80 ring-1 ring-inset ring-blue-500" : ""}`}
                  >
                    <td className="p-3 font-bold text-white sticky left-0 bg-slate-900 border-r border-slate-800/50 z-10">
                      {res.layoutName}
                    </td>
                    <td className="p-3 text-right font-mono text-blue-400 font-bold bg-slate-800/30">
                      {(s.layoutScore).toLocaleString(undefined, { maximumFractionDigits: 0 })}
                    </td>

                    {/* SFB Stats */}
                    <td className="p-3 text-right font-mono text-slate-300 border-l border-slate-800">{pct(s.statSfbBase, tBi)}%</td>
                    <td className="p-3 text-right font-mono text-slate-300">{pct(s.statSfbLat, tBi)}%</td>
                    <td className="p-3 text-right font-mono text-slate-300">{pct(s.statSfbLatWeak, tBi)}%</td>
                    <td className="p-3 text-right font-mono text-slate-300">{pct(s.statSfbDiag, tBi)}%</td>
                    <td className="p-3 text-right font-mono text-slate-300">{pct(s.statSfbLong, tBi)}%</td>
                    <td className="p-3 text-right font-mono text-slate-300">{pct(s.statSfbBot, tBi)}%</td>

                    {/* Mechanics Stats */}
                    <td className="p-3 text-right font-mono text-orange-300 border-l border-slate-800">{pct(s.statLsb, tBi)}%</td>
                    <td className="p-3 text-right font-mono text-yellow-300">{pct(s.statScis, tBi)}%</td>
                    <td className="p-3 text-right font-mono text-purple-300">{pct(s.statPinkyReach, tChar)}%</td>

                    {/* Flow Stats */}
                    <td className="p-3 text-right font-mono text-green-400 border-l border-slate-800">{pct(s.statRoll, tBi)}%</td>
                    <td className="p-3 text-right font-mono text-green-300">{pct(s.statRollTri, tTri)}%</td>
                    <td className="p-3 text-right font-mono text-blue-400">{pct(s.statRedir, tTri)}%</td>

                    <td className="p-3 text-right font-mono text-slate-500 border-l border-slate-800">
                      {(s.flowCost).toLocaleString(undefined, { maximumFractionDigits: 0 })}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

export default App;