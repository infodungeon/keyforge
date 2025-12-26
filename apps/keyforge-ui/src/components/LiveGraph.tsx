import { useMemo } from "react";

interface DataPoint {
  epoch: number;
  score: number;
}

interface Props {
  data: DataPoint[];
  width?: number;
  height?: number;
  color?: string;
}

export function LiveGraph({
  data,
  width = 300,
  height = 100,
  color = "#a855f7",
}: Props) {
  const points = useMemo(() => {
    if (data.length === 0) return "";

    const maxScore = Math.max(...data.map((d) => d.score)) * 1.05;
    const minScore = Math.min(...data.map((d) => d.score)) * 0.95;
    const maxEpoch = data[data.length - 1].epoch;
    const minEpoch = data[0].epoch;

    const rangeX = maxEpoch - minEpoch || 1;
    const rangeY = maxScore - minScore || 1;

    return data
      .map((d, i) => {
        const x = ((d.epoch - minEpoch) / rangeX) * width;
        const y = height - ((d.score - minScore) / rangeY) * height;
        return `${i === 0 ? "M" : "L"} ${x},${y}`;
      })
      .join(" ");
  }, [data, width, height]);

  if (data.length < 2)
    return (
      <div className="h-full w-full bg-slate-900/50 animate-pulse rounded" />
    );

  return (
    <div className="relative border border-slate-800 bg-slate-950/50 rounded-lg overflow-hidden">
      <svg
        width="100%"
        height="100%"
        viewBox={`0 0 ${width} ${height}`}
        preserveAspectRatio="none"
      >
        <line
          x1="0"
          y1={height * 0.25}
          x2={width}
          y2={height * 0.25}
          stroke="#1e293b"
          strokeWidth="1"
          strokeDasharray="4"
        />
        <line
          x1="0"
          y1={height * 0.5}
          x2={width}
          y2={height * 0.5}
          stroke="#1e293b"
          strokeWidth="1"
          strokeDasharray="4"
        />
        <line
          x1="0"
          y1={height * 0.75}
          x2={width}
          y2={height * 0.75}
          stroke="#1e293b"
          strokeWidth="1"
          strokeDasharray="4"
        />

        <path
          d={points}
          fill="none"
          stroke={color}
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
          vectorEffect="non-scaling-stroke"
        />
      </svg>

      <div className="absolute top-2 right-2 text-[10px] font-mono font-bold bg-slate-900/80 px-1.5 py-0.5 rounded text-white border border-slate-700">
        {data[data.length - 1].score.toFixed(1)}
      </div>
    </div>
  );
}
