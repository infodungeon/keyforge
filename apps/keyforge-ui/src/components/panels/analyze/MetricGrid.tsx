// apps/keyforge-ui/src/components/panels/analyze/MetricGrid.tsx

import { StatBox } from "../../Charts";
import { ValidationResult } from "../../../types";
import { DerivedStats } from "../../../utils";
import { MapMode } from "../../KeyboardMap";

interface Props {
    activeResult: ValidationResult;
    referenceResult: ValidationResult | null;
    mapMode: MapMode;
    includeThumbs: boolean;
    derivedStats: DerivedStats;
    showDiff: boolean;
}

export const MetricGrid = ({
    activeResult,
    referenceResult,
    mapMode,
    includeThumbs,
    derivedStats,
    showDiff,
}: Props) => {
    const total =
        mapMode === "penalty"
            ? activeResult.score.score
            : mapMode === "diff"
                ? 1.0 // Diff mode doesn't use standard normalization
                : includeThumbs
                    ? 100
                    : derivedStats.totalUsage;

    return (
        <div className="grid grid-cols-3 gap-2">
            <StatBox
                label="Travel/Key"
                val={activeResult.score.travel_per_key * 100}
                refVal={
                    referenceResult
                        ? referenceResult.score.travel_per_key * 100
                        : undefined
                }
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
                val={
                    mapMode === "penalty"
                        ? activeResult.score.sfb_penalty
                        : activeResult.score.sfb_total
                }
                total={total}
                showDiff={showDiff}
                color="text-red-400"
                suffix="%"
                precision={2}
            />
            <StatBox
                label="Scissor"
                val={
                    mapMode === "penalty"
                        ? activeResult.score.scissor_penalty
                        : activeResult.score.scissors
                }
                total={total}
                showDiff={showDiff}
                color="text-yellow-400"
                suffix="%"
                precision={2}
            />
            <StatBox
                label="Redir"
                val={
                    mapMode === "penalty"
                        ? activeResult.score.redir_penalty
                        : activeResult.score.redirects
                }
                total={total}
                showDiff={showDiff}
                color="text-blue-400"
                suffix="%"
                precision={2}
            />
            <StatBox
                label="Rolls"
                val={
                    mapMode === "penalty"
                        ? Math.abs(activeResult.score.roll_penalty)
                        : activeResult.score.rolls
                }
                total={total}
                showDiff={showDiff}
                color="text-green-400"
                suffix="%"
                precision={2}
                invertGood={true}
            />
        </div>
    );
};
