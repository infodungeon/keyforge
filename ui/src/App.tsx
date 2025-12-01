import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ValidationResult, SearchParams, ScoringWeights } from "./types";
import { KeyboardMap } from "./components/KeyboardMap";
import { OptimizerConfig } from "./components/OptimizerConfig";
import "./App.css";

// Defaults
const DEFAULT_PARAMS: SearchParams = {
  search_epochs: 10000,
  search_steps: 50000,
  search_patience: 500,
  search_patience_threshold: 0.1,
  temp_min: 0.08,
  temp_max: 1000.0,
  opt_limit_fast: 600,
  opt_limit_slow: 3000,
};

const DEFAULT_WEIGHTS: ScoringWeights = {
  penalty_sfb_base: 400,
  penalty_sfb_lateral: 65,
  penalty_sfb_lateral_weak: 160,
  penalty_sfb_diagonal: 240,
  penalty_sfb_long: 280,
  penalty_sfb_bottom: 45,
  penalty_sfr_bad_row: 25,
  penalty_sfr_weak_finger: 20,
  penalty_scissor: 25,
  penalty_lateral: 50,
  penalty_redirect: 65,
  penalty_skip: 20,
  penalty_hand_run: 5,
  bonus_inward_roll: 40,
  bonus_bigram_roll_in: 35,
  bonus_bigram_roll_out: 25,
  penalty_imbalance: 200,
  threshold_sfb_long_row_diff: 2,
  threshold_scissor_row_diff: 2,
};

interface SearchEvent {
  epoch: number;
  score: number;
  layout: string;
  ips: number;
}

function App() {
  // === STATE ===
  const [projectPath, setProjectPath] = useState("/home/robert/Documents/KeyboardLayouts/Data Driven Analysis/keyforge");
  const [keyboards, setKeyboards] = useState<string[]>([]);
  const [selectedKeyboard, setSelectedKeyboard] = useState("szr35");
  const [availableLayouts, setAvailableLayouts] = useState<Record<string, string>>({});

  const [layoutName, setLayoutName] = useState("Custom");
  const [layoutString, setLayoutString] = useState("");

  const [activeTab, setActiveTab] = useState<'analyze' | 'optimize'>('analyze');
  const [searchParams, setSearchParams] = useState<SearchParams>(DEFAULT_PARAMS);
  const [weights, setWeights] = useState<ScoringWeights>(DEFAULT_WEIGHTS);
  const [pinnedKeys, setPinnedKeys] = useState("");

  const [isSearching, setIsSearching] = useState(false);
  const [searchStats, setSearchStats] = useState<SearchEvent | null>(null);

  const [activeResult, setActiveResult] = useState<ValidationResult | null>(null);
  const [history, setHistory] = useState<ValidationResult[]>([]);

  // === INITIALIZATION ===
  useEffect(() => {
    // Load keyboard list on startup
    invoke<string[]>("cmd_list_keyboards", { basePath: projectPath })
      .then(kbs => {
        setKeyboards(kbs);
        // If our default isn't there, pick first
        if (!kbs.includes(selectedKeyboard) && kbs.length > 0) {
          setSelectedKeyboard(kbs[0]);
        }
      })
      .catch(console.error);

    // Listener
    const unlisten = listen<SearchEvent>('search-update', (event) => {
      setSearchStats(event.payload);
      setLayoutString(event.payload.layout);
    });

    return () => { unlisten.then(f => f()); };
  }, []);

  // === KEYBOARD LOADING ===
  useEffect(() => {
    if (!selectedKeyboard) return;
    async function loadKb() {
      try {
        await invoke("cmd_load_dataset", { basePath: projectPath, keyboardName: selectedKeyboard });
        const layouts = await invoke<Record<string, string>>("cmd_get_loaded_layouts");
        setAvailableLayouts(layouts);

        // Auto-select Qwerty if available
        const defaultLayout = layouts["Qwerty"] || Object.values(layouts)[0] || "";
        setLayoutString(defaultLayout);
        setLayoutName(layouts["Qwerty"] ? "Qwerty" : "Custom");

        // Validate immediately
        if (defaultLayout) validateInternal("Current", defaultLayout);
      } catch (e) {
        console.error("Failed to load keyboard:", e);
      }
    }
    loadKb();
  }, [selectedKeyboard, projectPath]);

  const validateInternal = async (name: string, str: string) => {
    try {
      const res = await invoke<ValidationResult>("cmd_validate_layout", {
        layoutStr: str,
        weights: activeTab === 'optimize' ? weights : null // Use custom weights only if optimizing
      });
      const finalRes = { ...res, layoutName: name };
      setActiveResult(finalRes);
      setHistory(prev => [finalRes, ...prev.filter(x => x.layoutName !== name)]);
    } catch (e) { console.error(e); }
  };

  const handleStartSearch = async () => {
    setIsSearching(true);
    setSearchStats(null);
    try {
      await invoke("cmd_start_search", {
        request: {
          pinned_keys: pinnedKeys,
          search_params: searchParams,
          weights: weights
        }
      });
    } catch (e) {
      console.error(e);
    }
    setIsSearching(false);
    validateInternal("Optimized", layoutString);
  };

  const handlePresetChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const name = e.target.value;
    if (availableLayouts[name]) {
      setLayoutName(name);
      setLayoutString(availableLayouts[name]);
      validateInternal(name, availableLayouts[name]);
    } else {
      setLayoutName("Custom");
    }
  };

  const pct = (val: number, total: number) => total ? ((val / total) * 100).toFixed(2) : "0.00";

  return (
    <div className="h-screen bg-slate-950 text-slate-200 font-sans flex flex-col overflow-hidden">

      {/* HEADER */}
      <header className="flex items-center gap-4 border-b border-slate-800 bg-slate-900 px-6 py-3 shrink-0">
        <h1 className="text-2xl font-black tracking-tight text-white">
          KEY<span className="text-blue-500">FORGE</span>
        </h1>

        <div className="flex gap-2 items-center ml-8">
          <label className="text-[10px] font-bold text-slate-500 uppercase">Keyboard</label>
          <select
            className="bg-slate-800 border border-slate-700 text-xs rounded px-2 py-1 outline-none focus:border-blue-500"
            value={selectedKeyboard}
            onChange={(e) => setSelectedKeyboard(e.target.value)}
            disabled={isSearching}
          >
            {keyboards.map(k => <option key={k} value={k}>{k}</option>)}
          </select>
        </div>

        <div className="flex gap-2 ml-auto">
          <button
            onClick={() => setActiveTab('analyze')}
            className={`px-4 py-1 text-xs font-bold rounded-full transition ${activeTab === 'analyze' ? 'bg-blue-600 text-white' : 'bg-slate-800 text-slate-400 hover:bg-slate-700'}`}
          >
            ANALYZER
          </button>
          <button
            onClick={() => setActiveTab('optimize')}
            className={`px-4 py-1 text-xs font-bold rounded-full transition ${activeTab === 'optimize' ? 'bg-purple-600 text-white' : 'bg-slate-800 text-slate-400 hover:bg-slate-700'}`}
          >
            OPTIMIZER
          </button>
        </div>
      </header>

      {/* MAIN CONTENT AREA */}
      <div className="flex flex-1 overflow-hidden">

        {/* LEFT PANEL (Config/Stats) */}
        <div className="w-80 bg-slate-900 border-r border-slate-800 flex flex-col shrink-0">

          {/* LAYOUT INPUT AREA */}
          <div className="p-4 border-b border-slate-800">
            <div className="mb-2">
              <label className="text-[10px] font-bold text-slate-500 uppercase">Layout Preset</label>
              <select
                className="w-full bg-slate-800 border border-slate-700 rounded px-2 py-1 text-xs mt-1"
                value={availableLayouts[layoutName] === layoutString ? layoutName : "Custom"}
                onChange={handlePresetChange}
                disabled={isSearching}
              >
                <option value="Custom">Custom</option>
                {Object.keys(availableLayouts).sort().map(k => <option key={k} value={k}>{k}</option>)}
              </select>
            </div>

            <div className="mb-2">
              <label className="text-[10px] font-bold text-slate-500 uppercase flex justify-between">
                <span>Mapping</span>
                <span>{layoutString.length} keys</span>
              </label>
              <input
                className="w-full bg-slate-800 border border-slate-700 rounded px-2 py-1 font-mono text-sm mt-1 uppercase"
                value={layoutString}
                onChange={(e) => setLayoutString(e.target.value.toUpperCase())}
                readOnly={isSearching}
              />
            </div>

            {activeTab === 'analyze' && (
              <button
                onClick={() => validateInternal("Custom", layoutString)}
                className="w-full bg-blue-600 hover:bg-blue-500 text-white font-bold py-2 rounded text-xs mt-2"
              >
                ANALYZE LAYOUT
              </button>
            )}
          </div>

          {/* TAB CONTENT */}
          <div className="flex-1 overflow-hidden p-4">
            {activeTab === 'optimize' ? (
              <div className="flex flex-col h-full">
                <div className="mb-4">
                  <label className="text-[10px] font-bold text-slate-500 uppercase">Pinned Keys (idx:char)</label>
                  <input
                    className="w-full bg-slate-800 border border-slate-700 rounded px-2 py-1 font-mono text-xs mt-1"
                    value={pinnedKeys}
                    onChange={(e) => setPinnedKeys(e.target.value)}
                    placeholder="0:q, 10:a"
                    disabled={isSearching}
                  />
                </div>

                <OptimizerConfig
                  weights={weights}
                  searchParams={searchParams}
                  onWeightsChange={setWeights}
                  onParamsChange={setSearchParams}
                />

                <div className="mt-4 pt-4 border-t border-slate-800">
                  {!isSearching ? (
                    <button
                      onClick={handleStartSearch}
                      className="w-full bg-purple-600 hover:bg-purple-500 text-white font-bold py-3 rounded shadow-lg animate-pulse"
                    >
                      START OPTIMIZATION
                    </button>
                  ) : (
                    <div className="flex flex-col gap-2">
                      <div className="flex justify-between text-xs font-mono">
                        <span className="text-slate-400">Epoch:</span>
                        <span className="text-white">{searchStats?.epoch}</span>
                      </div>
                      <div className="flex justify-between text-xs font-mono">
                        <span className="text-slate-400">Speed:</span>
                        <span className="text-white">{searchStats?.ips.toFixed(2)} M/s</span>
                      </div>
                      <div className="bg-slate-800 rounded p-2 text-center border border-green-500/30">
                        <span className="text-xl font-bold text-green-400 font-mono">{(searchStats?.score || 0).toFixed(0)}</span>
                      </div>
                      <button
                        onClick={() => invoke("cmd_stop_search")}
                        className="w-full bg-red-600 hover:bg-red-500 text-white font-bold py-2 rounded"
                      >
                        STOP
                      </button>
                    </div>
                  )}
                </div>
              </div>
            ) : (
              <div className="text-slate-500 text-xs italic text-center mt-10">
                Select a layout above to view statistics.
              </div>
            )}
          </div>
        </div>

        {/* RIGHT PANEL (Visualizer & Table) */}
        <div className="flex-1 flex flex-col bg-slate-950 overflow-hidden">

          {/* VISUALIZER */}
          <div className="flex-1 flex items-center justify-center bg-slate-950 relative">
            <div className="absolute inset-0 flex items-center justify-center opacity-5 pointer-events-none">
              <span className="text-9xl font-black text-slate-800 select-none">KF</span>
            </div>

            <div className="z-10 bg-slate-900/80 backdrop-blur border border-slate-800 rounded-xl p-6 shadow-2xl">
              <h2 className="text-center text-xl font-black text-white mb-4 tracking-wide uppercase">
                {isSearching ? "Optimizing..." : (activeResult?.layoutName || layoutName)}
              </h2>
              <KeyboardMap
                geometry={activeResult?.geometry}
                layoutString={layoutString}
                heatmap={isSearching ? undefined : activeResult?.heatmap}
              />
            </div>
          </div>

          {/* HISTORY TABLE */}
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
                      onClick={() => { if (!isSearching) { setActiveResult(res); setLayoutName(res.layoutName); setLayoutString(res.layoutName === "Custom" ? layoutString : availableLayouts[res.layoutName] || layoutString) } }}
                      className="cursor-pointer hover:bg-slate-800"
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
        </div>
      </div>
    </div>
  );
}

export default App;