import { SearchParams, ScoringWeights } from "../types";

interface Props {
  weights: ScoringWeights;
  searchParams: SearchParams;
  onWeightsChange: (w: ScoringWeights) => void;
  onParamsChange: (p: SearchParams) => void;
}

export function OptimizerConfig({ weights, searchParams, onWeightsChange, onParamsChange }: Props) {

  const handleWeightChange = (key: string, val: string) => {
    const num = parseFloat(val);
    if (!isNaN(num)) {
      onWeightsChange({ ...weights, [key]: num });
    }
  };

  const handleParamChange = (key: string, val: string) => {
    const num = parseFloat(val);
    if (!isNaN(num)) {
      onParamsChange({ ...searchParams, [key]: num });
    }
  };

  return (
    <div className="space-y-4">
      {/* Search Parameters Section */}
      <div>
        <h3 className="text-xs font-bold text-slate-400 uppercase mb-2 border-b border-slate-700 pb-1">
          Algorithm Params
        </h3>
        <div className="grid grid-cols-2 gap-2">
          <div>
            <label className="text-[9px] text-slate-500 uppercase block">Epochs</label>
            <input
              type="number"
              className="w-full bg-slate-950 border border-slate-700 rounded px-2 py-1 text-xs text-slate-200 outline-none focus:border-purple-500"
              value={searchParams.search_epochs}
              onChange={(e) => handleParamChange("search_epochs", e.target.value)}
            />
          </div>
          <div>
            <label className="text-[9px] text-slate-500 uppercase block">Steps/Epoch</label>
            <input
              type="number"
              className="w-full bg-slate-950 border border-slate-700 rounded px-2 py-1 text-xs text-slate-200 outline-none focus:border-purple-500"
              value={searchParams.search_steps}
              onChange={(e) => handleParamChange("search_steps", e.target.value)}
            />
          </div>
          <div>
            <label className="text-[9px] text-slate-500 uppercase block">Min Temp</label>
            <input
              type="number"
              step="0.01"
              className="w-full bg-slate-950 border border-slate-700 rounded px-2 py-1 text-xs text-slate-200 outline-none focus:border-purple-500"
              value={searchParams.temp_min}
              onChange={(e) => handleParamChange("temp_min", e.target.value)}
            />
          </div>
          <div>
            <label className="text-[9px] text-slate-500 uppercase block">Max Temp</label>
            <input
              type="number"
              className="w-full bg-slate-950 border border-slate-700 rounded px-2 py-1 text-xs text-slate-200 outline-none focus:border-purple-500"
              value={searchParams.temp_max}
              onChange={(e) => handleParamChange("temp_max", e.target.value)}
            />
          </div>
        </div>
      </div>

      {/* Weights Section */}
      <div>
        <h3 className="text-xs font-bold text-slate-400 uppercase mb-2 border-b border-slate-700 pb-1">
          Scoring Weights
        </h3>
        <div className="grid grid-cols-1 gap-2 max-h-40 overflow-y-auto pr-1 custom-scrollbar">
          {Object.entries(weights).map(([key, val]) => (
            <div key={key} className="flex items-center justify-between">
              <label className="text-[9px] text-slate-500 uppercase truncate w-32" title={key}>
                {key.replace("penalty_", "").replace("bonus_", "").replace(/_/g, " ")}
              </label>
              <input
                type="number"
                className="w-16 bg-slate-950 border border-slate-700 rounded px-2 py-1 text-xs text-slate-200 text-right outline-none focus:border-purple-500"
                value={val}
                onChange={(e) => handleWeightChange(key, e.target.value)}
              />
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}