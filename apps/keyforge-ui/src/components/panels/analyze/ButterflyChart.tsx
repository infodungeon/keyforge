// apps/keyforge-ui/src/components/panels/analyze/ButterflyChart.tsx

interface Props {
    left: number[];
    right: number[];
}

export const ButterflyChart = ({ left, right }: Props) => {
    // Finger indices: 0=Thumb, 1=Index, 2=Mid, 3=Ring, 4=Pinky
    // Labels for the rows
    const labels = ["Pinky", "Ring", "Mid", "Index", "Thumb"];

    // We map the display rows (0..4) to the actual finger indices
    // Top row (0) = Pinky (index 4)
    // Bottom row (4) = Thumb (index 0)
    const mapRowToFingerIdx = (row: number) => 4 - row;

    const Row = ({ rowIdx }: { rowIdx: number }) => {
        const fingerIdx = mapRowToFingerIdx(rowIdx);
        const lVal = left[fingerIdx] || 0;
        const rVal = right[fingerIdx] || 0;
        const label = labels[rowIdx];

        // Colors per finger index (0..4)
        const colors = [
            "bg-slate-500", // Thumb
            "bg-green-500", // Index
            "bg-blue-500", // Mid
            "bg-purple-500", // Ring
            "bg-pink-500", // Pinky
        ];
        const color = colors[fingerIdx];

        return (
            <div className="flex items-center gap-2 text-[9px] mb-1.5">
                {/* Left Side (Right Aligned) */}
                <div className="flex-1 flex justify-end items-center gap-2">
                    <span className="text-slate-500 font-mono w-8 text-right">
                        {lVal.toFixed(1)}%
                    </span>
                    <div className="h-1.5 w-24 bg-slate-800/50 rounded-l-full overflow-hidden flex justify-end">
                        <div
                            className={`h-full ${color} opacity-80`}
                            style={{ width: `${Math.min(100, lVal * 4)}%` }}
                        />
                    </div>
                </div>

                {/* Center Label */}
                <div className="w-10 text-center text-slate-600 font-bold uppercase text-[8px]">
                    {label}
                </div>

                {/* Right Side (Left Aligned) */}
                <div className="flex-1 flex justify-start items-center gap-2">
                    <div className="h-1.5 w-24 bg-slate-800/50 rounded-r-full overflow-hidden flex justify-start">
                        <div
                            className={`h-full ${color} opacity-80`}
                            style={{ width: `${Math.min(100, rVal * 4)}%` }}
                        />
                    </div>
                    <span className="text-slate-500 font-mono w-8 text-left">
                        {rVal.toFixed(1)}%
                    </span>
                </div>
            </div>
        );
    };

    return (
        <div className="flex flex-col w-full">
            {[0, 1, 2, 3, 4].map((i) => (
                <Row key={i} rowIdx={i} />
            ))}
        </div>
    );
};
