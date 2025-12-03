import { ScoringWeights, SearchParams } from "../../types";
import { OptimizerConfig } from "../OptimizerConfig";
import { Card } from "../ui/Card";
import { Label } from "../ui/Label";
import { Input } from "../ui/Input";

interface Props {
    weights: ScoringWeights | null;
    searchParams: SearchParams | null;
    pinnedKeys: string;
    localWorkerEnabled: boolean;
    setWeights: (w: ScoringWeights) => void;
    setSearchParams: (s: SearchParams) => void;
    setPinnedKeys: (s: string) => void;
    toggleWorker: (b: boolean) => void;
}

export function OptimizePanel({ 
    weights, searchParams, pinnedKeys, localWorkerEnabled,
    setWeights, setSearchParams, setPinnedKeys, toggleWorker 
}: Props) {
    
    if (!weights || !searchParams) return null;

    return (
        <div className="space-y-6">
            <Card className="bg-slate-800/50 p-3">
                <Label>Pinned Keys</Label>
                <Input
                    value={pinnedKeys}
                    onChange={e => setPinnedKeys(e.target.value)}
                    placeholder="0:Q, 10:A..."
                    mono
                />
            </Card>

            <OptimizerConfig
                weights={weights}
                searchParams={searchParams}
                onWeightsChange={setWeights}
                onParamsChange={setSearchParams}
            />

            <Card className="p-3 flex items-center justify-between">
                <div>
                    <div className="text-xs font-bold text-slate-300">Local Worker</div>
                    <div className="text-[9px] text-slate-500">Donate CPU</div>
                </div>
                <input
                    type="checkbox"
                    checked={localWorkerEnabled}
                    onChange={e => toggleWorker(e.target.checked)}
                    className="accent-purple-500 h-4 w-4"
                />
            </Card>
        </div>
    );
}