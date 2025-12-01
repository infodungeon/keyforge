import { useState } from "react";
import { ScoringWeights, SearchParams } from "../types";

interface Props {
  weights: ScoringWeights;
  searchParams: SearchParams;
  onWeightsChange: (w: ScoringWeights) => void;
  onParamsChange: (p: SearchParams) => void;
}

const InputGroup = ({ label, children }: { label: string, children: React.ReactNode }) => (
  <div className="bg-slate-800/50 p-3 rounded-lg border border-slate-700 mb-2">
    <h3 className="text-xs font-bold text-slate-400 uppercase mb-3 border-b border-slate-700 pb-1">{label}</h3>
    <div className="grid grid-cols-2 gap-x-4 gap-y-2">
      {children}
    </div>
  </div>
);

const NumberInput = ({ label, value, onChange }: { label: string, value: number, onChange: (v: number) => void }) => (
  <div className="flex flex-col">
    <label className="text-[10px] text-slate-500 mb-0.5">{label}</label>
    <input
      type="number"
      className="bg-slate-900 border border-slate-700 rounded px-2 py-1 text-xs text-white focus:border-blue-500 outline-none"
      value={value}
      onChange={(e) => onChange(parseFloat(e.target.value))}
    />
  </div>
);

export function OptimizerConfig({ weights, searchParams, onWeightsChange, onParamsChange }: Props) {
  const updateW = (key: keyof ScoringWeights, val: number) => onWeightsChange({ ...weights, [key]: val });
  const updateP = (key: keyof SearchParams, val: number) => onParamsChange({ ...searchParams, [key]: val });

  return (
    <div className="h-full overflow-y-auto pr-2 custom-scrollbar">
      <InputGroup label="Annealing Parameters">
        <NumberInput label="Min Temp" value={searchParams.temp_min} onChange={(v) => updateP('temp_min', v)} />
        <NumberInput label="Max Temp" value={searchParams.temp_max} onChange={(v) => updateP('temp_max', v)} />
        <NumberInput label="Epochs" value={searchParams.search_epochs} onChange={(v) => updateP('search_epochs', v)} />
        <NumberInput label="Steps/Epoch" value={searchParams.search_steps} onChange={(v) => updateP('search_steps', v)} />
        <NumberInput label="Limit (Fast)" value={searchParams.opt_limit_fast} onChange={(v) => updateP('opt_limit_fast', v)} />
        <NumberInput label="Limit (Slow)" value={searchParams.opt_limit_slow} onChange={(v) => updateP('opt_limit_slow', v)} />
      </InputGroup>

      <InputGroup label="Bigram Penalties (SFB)">
        <NumberInput label="Base SFB" value={weights.penalty_sfb_base} onChange={(v) => updateW('penalty_sfb_base', v)} />
        <NumberInput label="Lateral SFB" value={weights.penalty_sfb_lateral} onChange={(v) => updateW('penalty_sfb_lateral', v)} />
        <NumberInput label="Weak Finger" value={weights.penalty_sfb_lateral_weak} onChange={(v) => updateW('penalty_sfb_lateral_weak', v)} />
        <NumberInput label="Diagonal" value={weights.penalty_sfb_diagonal} onChange={(v) => updateW('penalty_sfb_diagonal', v)} />
        <NumberInput label="Long Jump" value={weights.penalty_sfb_long} onChange={(v) => updateW('penalty_sfb_long', v)} />
        <NumberInput label="Bottom Row" value={weights.penalty_sfb_bottom} onChange={(v) => updateW('penalty_sfb_bottom', v)} />
      </InputGroup>

      <InputGroup label="Physics & Flow">
        <NumberInput label="Scissors" value={weights.penalty_scissor} onChange={(v) => updateW('penalty_scissor', v)} />
        <NumberInput label="Lateral Stretch" value={weights.penalty_lateral} onChange={(v) => updateW('penalty_lateral', v)} />
        <NumberInput label="Redirects" value={weights.penalty_redirect} onChange={(v) => updateW('penalty_redirect', v)} />
        <NumberInput label="Bad Redirects" value={weights.penalty_skip} onChange={(v) => updateW('penalty_skip', v)} />
        <NumberInput label="Roll Bonus (In)" value={weights.bonus_inward_roll} onChange={(v) => updateW('bonus_inward_roll', v)} />
        <NumberInput label="Hand Imbalance" value={weights.penalty_imbalance} onChange={(v) => updateW('penalty_imbalance', v)} />
      </InputGroup>
    </div>
  );
}