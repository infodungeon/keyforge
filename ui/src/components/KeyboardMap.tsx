import { KeyboardGeometry } from "../types";

interface Props {
  geometry?: KeyboardGeometry; // Changed to optional (?)
  layoutString: string;
  heatmap?: number[];
}

export function KeyboardMap({ geometry, layoutString, heatmap }: Props) {
  // Safe guard handles undefined geometry
  if (!geometry || !geometry.keys) return null;

  const maxX = Math.max(...geometry.keys.map((k) => k.x)) + 1;
  const maxY = Math.max(...geometry.keys.map((k) => k.y)) + 1;
  const UNIT = 55;
  const GAP = 4;

  const getHeatColor = (intensity: number) => {
    const r = 255;
    const g = Math.floor(255 - (intensity * 180));
    const b = Math.floor(255 - (intensity * 180));
    return `rgb(${r}, ${g}, ${b})`;
  };

  return (
    <div className="flex justify-center items-center w-full p-8 bg-white dark:bg-gray-800/50 rounded-2xl border border-gray-200 dark:border-gray-700 shadow-sm">
      <svg
        width={maxX * UNIT}
        height={maxY * UNIT}
        viewBox={`-5 -5 ${maxX * UNIT + 10} ${maxY * UNIT + 10}`}
        style={{ maxWidth: "100%", maxHeight: "100%" }}
      >
        {geometry.keys.map((key, index) => {
          const char = layoutString[index];
          if (!char) return null;

          const intensity = heatmap && heatmap.length > index ? heatmap[index] : 0;
          const fillColor = heatmap ? getHeatColor(intensity) : "#e5e7eb";

          return (
            <g key={index} transform={`translate(${key.x * UNIT}, ${key.y * UNIT})`}>
              <rect
                width={UNIT - GAP}
                height={UNIT - GAP}
                rx={6}
                fill={fillColor}
                className="stroke-slate-300 dark:stroke-slate-600 stroke-[1px] transition-all duration-500"
              />
              <text
                x={(UNIT - GAP) / 2}
                y={(UNIT - GAP) / 2 + 6}
                textAnchor="middle"
                alignmentBaseline="middle"
                className="fill-slate-900 dark:fill-slate-800 font-black text-xl pointer-events-none uppercase"
                style={{ textShadow: "0px 1px 0px rgba(255,255,255,0.5)" }}
              >
                {char}
              </text>
            </g>
          );
        })}
      </svg>
    </div>
  );
}