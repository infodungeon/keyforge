// ===== keyforge/ui/src/components/KeyboardMap.tsx =====
import { KeyboardGeometry } from "../types";
import { keycodeService } from "../utils";

const UNIT = 54;
const GAP = 4;

export type MapMode = 'frequency' | 'penalty';

function getHeatmapColor(intensity: number, mode: MapMode): string {
  if (intensity <= 0) return "rgba(30, 41, 59, 1)"; // Slate-800

  if (mode === 'penalty') {
    // Penalty Scale: Yellow -> Red -> Dark Red
    // 0.0 -> Yellow (60)
    // 1.0 -> Red (0)
    const h = (1.0 - intensity) * 60;
    return `hsla(${h}, 90%, 50%, 0.8)`;
  } else {
    // Frequency Scale: Blue -> Green -> Yellow -> Red (Standard Heatmap)
    const h = (1.0 - intensity) * 240;
    return `hsla(${h}, 70%, 50%, 0.8)`;
  }
}

function getKeyStyle(
  index: number,
  dataMap: number[] | undefined,
  maxVal: number,
  isSelected: boolean,
  isEditing: boolean,
  isActive: boolean,
  mode: MapMode
) {
  if (isActive) {
    return { fill: "#22c55e", stroke: "#15803d", strokeWidth: 2, text: "#ffffff" };
  }

  let fill = "rgba(30, 41, 59, 1)";
  let stroke = "rgb(2, 6, 23)";
  let strokeWidth = 2;
  let text = "#94a3b8";

  if (dataMap && dataMap[index] && dataMap[index] > 0) {
    const val = dataMap[index];
    // Non-linear scaling for better contrast
    const intensity = Math.pow(val / maxVal, 0.7);
    fill = getHeatmapColor(intensity, mode);
    text = "#ffffff";
  }

  if (isSelected) {
    stroke = "#3b82f6";
    strokeWidth = 3;
    if (!dataMap || !dataMap[index]) fill = "rgba(51, 65, 85, 1)";
  }

  if (isSelected && isEditing) {
    fill = "#2563eb";
    stroke = "#60a5fa";
    strokeWidth = 4;
    text = "#ffffff";
  }

  return { fill, stroke, strokeWidth, text };
}

interface KeyboardMapProps {
  geometry?: KeyboardGeometry;
  layoutString: string;
  heatmap?: number[];
  className?: string;
  selectedKeyIndex?: number | null;
  isEditing?: boolean;
  onKeyClick?: (index: number) => void;
  onKeyPointerDown?: (index: number) => void;
  onKeyPointerUp?: (index: number) => void;
  activeKeyIds?: Set<string>;
  mode?: MapMode; // ADDED
}

export function KeyboardMap({
  geometry, layoutString, heatmap, className = "",
  selectedKeyIndex, isEditing = false,
  onKeyClick, onKeyPointerDown, onKeyPointerUp,
  activeKeyIds,
  mode = 'frequency' // Default
}: KeyboardMapProps) {

  if (!geometry || !geometry.keys) return (
    <div className="flex flex-col items-center justify-center h-full text-slate-600 font-mono text-xs gap-2">
      <div className="animate-pulse bg-slate-800/50 h-32 w-64 rounded-lg border border-slate-700/50"></div>
      <div>NO GEOMETRY LOADED</div>
    </div>
  );

  const maxX = Math.max(...geometry.keys.map((k) => k.x + (k.w || 1)));
  const maxY = Math.max(...geometry.keys.map((k) => k.y + (k.h || 1)));

  // Map data is already normalized 0-1 by backend, but we take max just in case
  const maxVal = heatmap ? Math.max(...heatmap, 1.0) : 1.0;

  const tokens = layoutString.trim().split(/\s+/);

  return (
    <div className={`flex items-center justify-center w-full h-full overflow-hidden ${className}`}>
      <svg
        width="100%"
        height="100%"
        viewBox={`-10 -10 ${maxX * UNIT + 20} ${maxY * UNIT + 20}`}
        preserveAspectRatio="xMidYMid meet"
        style={{ display: 'block' }}
      >
        {geometry.keys.map((key, index) => {
          const keyId = key.id || "";
          const isActive = activeKeyIds ? activeKeyIds.has(keyId) : false;

          const style = getKeyStyle(
            index, heatmap, maxVal,
            selectedKeyIndex === index,
            selectedKeyIndex === index && isEditing,
            isActive,
            mode
          );

          let label = "";
          if (layoutString && tokens.length > 0) {
            const token = tokens[index] || "";
            label = keycodeService.getVisualLabel(token);
          } else {
            label = keycodeService.getVisualLabel(keyId);
          }

          const w = key.w || 1;
          const h = key.h || 1;
          const yOffset = isActive ? 2 : 0;
          const transform = `translate(${key.x * UNIT}px, ${key.y * UNIT + yOffset}px)`;

          return (
            <g
              key={index}
              style={{ transform, transition: 'transform 50ms ease-out' }}
              onPointerDown={(e) => { e.preventDefault(); onKeyPointerDown && onKeyPointerDown(index); }}
              onPointerUp={(e) => { e.preventDefault(); onKeyPointerUp && onKeyPointerUp(index); }}
              onPointerLeave={() => { onKeyPointerUp && onKeyPointerUp(index); }}
              onClick={(e) => { e.stopPropagation(); onKeyClick && onKeyClick(index); }}
              className="cursor-pointer select-none"
            >
              <rect
                width={w * UNIT - GAP}
                height={h * UNIT - GAP}
                rx={6}
                fill={style.fill}
                stroke={style.stroke}
                strokeWidth={style.strokeWidth}
                className="transition-colors duration-200"
              />
              <text
                x={(w * UNIT - GAP) / 2}
                y={(h * UNIT - GAP) / 2 + 7}
                textAnchor="middle"
                alignmentBaseline="middle"
                fill={style.text}
                fontSize={label.length > 2 ? 12 : 18}
                fontWeight="bold"
                className="pointer-events-none font-mono tracking-tight"
                style={{ textShadow: "0px 1px 2px rgba(0,0,0,0.5)" }}
              >
                {label}
              </text>
            </g>
          );
        })}
      </svg>
    </div>
  );
}