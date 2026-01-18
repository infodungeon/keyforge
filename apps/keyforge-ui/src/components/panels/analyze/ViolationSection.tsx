// apps/keyforge-ui/src/components/panels/analyze/ViolationSection.tsx

import { ViolationTable } from "./ViolationTable";
import { ValidationResult } from "../../../types";

import { MapMode } from "../../KeyboardMap";

interface Props {
    activeResult: ValidationResult;
    mapMode: MapMode;
}

export const ViolationSection = ({ activeResult, mapMode }: Props) => {
    return (
        <div className="mt-4 pt-4 border-t border-slate-800 animate-in fade-in slide-in-from-top-2">
            <ViolationTable
                title="Top SFBs"
                items={activeResult.score.top_sfbs}
                color="text-red-400"
                mapMode={mapMode}
                totalScore={activeResult.score.score}
            />
            <ViolationTable
                title="Top Scissors"
                items={activeResult.score.top_scissors}
                color="text-yellow-400"
                mapMode={mapMode}
                totalScore={activeResult.score.score}
            />
            <ViolationTable
                title="Top Redirects"
                items={activeResult.score.top_redirs}
                color="text-blue-400"
                mapMode={mapMode}
                totalScore={activeResult.score.score}
            />
        </div>
    );
};
