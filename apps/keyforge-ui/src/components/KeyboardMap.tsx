import { memo, useMemo } from "react";
import { KeyboardGeometry } from "../types";
import { keycodeService } from "../utils";

const UNIT = 54;
const GAP = 4;

export type MapMode = "frequency" | "penalty" | "diff";

function getHeatmapColor(val: number, maxVal: number, mode: MapMode): string {
  if (mode === "diff") {
    if (Math.abs(val) < 0.01) return "rgba(30, 41, 59, 1)";
    const intensity = Math.min(Math.abs(val) / (maxVal || 1), 1.0);
    const alpha = 0.3 + intensity * 0.7;
    return val < 0
      ? `hsla(142, 70%, 45%, ${alpha})`
      : `hsla(0, 90%, 60%, ${alpha})`;
  }

  if (val <= 0) return "rgba(30, 41, 59, 1)";
  const intensity = Math.pow(val / maxVal, 0.7);

  return mode === "frequency"
    ? `hsla(220, 90%, 60%, ${intensity * 0.9})`
    : `hsla(0, 90%, 60%, ${intensity * 0.9})`;
}

interface KeyboardMapProps {
  geometry?: KeyboardGeometry;
  layoutString: string;
  ghostLayoutString?: string;
  heatmap?: number[];
  className?: string;
  selectedKeyIndex?: number | null;
  isEditing?: boolean;
  onKeyClick?: (index: number) => void;
  onKeyPointerDown?: (index: number) => void;
  onKeyPointerUp?: (index: number) => void;
  activeKeyIds?: Set<string>;
  mode?: MapMode;
}

// UI-010: Memoize the component to prevent re-renders during typing/optimization
export const KeyboardMap = memo(function KeyboardMap({
  geometry,
  layoutString,
  ghostLayoutString,
  heatmap,
  className = "",
  selectedKeyIndex,
  isEditing = false,
  onKeyClick,
  onKeyPointerDown,
  onKeyPointerUp,
  activeKeyIds,
  mode = "frequency",
}: KeyboardMapProps) {
  if (!geometry || !geometry.keys)
    return (
      <div className="flex flex-col items-center justify-center h-full text-slate-600 font-mono text-xs gap-2">
        <div className="animate-pulse bg-slate-800/50 h-32 w-64 rounded-lg border border-slate-700/50"></div>
        <div>NO GEOMETRY LOADED</div>
      </div>
    );

  // UI-012: Memoize expensive calculations
  const { maxX, maxY, maxVal, tokens, ghostTokens } = useMemo(() => {
    const maxX = Math.max(...geometry.keys.map((k) => k.x + (k.w || 1)));
    const maxY = Math.max(...geometry.keys.map((k) => k.y + (k.h || 1)));

    let maxVal = 1.0;
    if (heatmap) {
      if (mode === "diff") {
        maxVal = Math.max(...heatmap.map(Math.abs), 0.1);
      } else {
        maxVal = Math.max(...heatmap, 1.0);
      }
    }

    const tokens = layoutString.trim().split(/\s+/);
    const ghostTokens = ghostLayoutString
      ? ghostLayoutString
          .trim()
          .split(/\s+/)
          .map((t) => keycodeService.getVisualLabel(t))
      : [];

    return { maxX, maxY, maxVal, tokens, ghostTokens };
  }, [geometry, layoutString, ghostLayoutString, heatmap, mode]);

  return (
    <div
      className={`flex items-center justify-center w-full h-full overflow-hidden ${className}`}
    >
      <svg
        width="100%"
        height="100%"
        viewBox={`-10 -10 ${maxX * UNIT + 20} ${maxY * UNIT + 20}`}
        preserveAspectRatio="xMidYMid meet"
        style={{ display: "block" }}
      >
        {geometry.keys.map((key, index) => {
          const keyId = key.id || "";
          const isActive = activeKeyIds ? activeKeyIds.has(keyId) : false;
          const isSelected = selectedKeyIndex === index;

          // Style calculation is fast, but we can inline it for clarity
          let fill = "rgba(30, 41, 59, 1)";
          let stroke = "rgb(2, 6, 23)";
          let strokeWidth = 2;
          let textColor = "#94a3b8";

          if (isActive) {
            fill = "#22c55e";
            stroke = "#15803d";
            textColor = "#ffffff";
          } else if (heatmap && heatmap[index] !== undefined) {
            const val = heatmap[index];
            if (Math.abs(val) > 0.001) {
              fill = getHeatmapColor(val, maxVal, mode);
              textColor = "#ffffff";
            }
          }

          if (isSelected) {
            stroke = "#3b82f6";
            strokeWidth = 3;
            if (isEditing) {
              fill = "#2563eb";
              stroke = "#60a5fa";
              strokeWidth = 4;
              textColor = "#ffffff";
            }
          }

          let label = "";
          if (layoutString && tokens.length > 0) {
            const token = tokens[index] || "";
            label = keycodeService.getVisualLabel(token);
          } else {
            label = keycodeService.getVisualLabel(keyId);
          }

          const ghostLabel = ghostTokens[index];
          const hasGhost = ghostLabel && ghostLabel !== label;

          const w = key.w || 1;
          const h = key.h || 1;
          const yOffset = isActive ? 2 : 0;

          const r = key.r ?? 0;
          const rx = (key.rx ?? 0) * UNIT;
          const ry = (key.ry ?? 0) * UNIT;

          const rotateStr = r !== 0 ? `rotate(${r}, ${rx}, ${ry})` : "";
          const transform = `translate(${key.x * UNIT}px, ${key.y * UNIT + yOffset}px) ${rotateStr}`;

          return (
            <g
              key={index}
              style={{ transform, transition: "transform 50ms ease-out" }}
              onPointerDown={(e) => {
                e.preventDefault();
                onKeyPointerDown && onKeyPointerDown(index);
              }}
              onPointerUp={(e) => {
                e.preventDefault();
                onKeyPointerUp && onKeyPointerUp(index);
              }}
              onPointerLeave={() => {
                onKeyPointerUp && onKeyPointerUp(index);
              }}
              onClick={(e) => {
                e.stopPropagation();
                onKeyClick && onKeyClick(index);
              }}
              className="cursor-pointer select-none"
            >
              <rect
                width={w * UNIT - GAP}
                height={h * UNIT - GAP}
                rx={6}
                fill={fill}
                stroke={stroke}
                strokeWidth={strokeWidth}
                className="transition-colors duration-200"
              />
              <text
                x={(w * UNIT - GAP) / 2}
                y={(h * UNIT - GAP) / 2 + 7}
                textAnchor="middle"
                alignmentBaseline="middle"
                fill={textColor}
                fontSize={label.length > 2 ? 12 : 18}
                fontWeight="bold"
                className="pointer-events-none font-mono tracking-tight"
                style={{ textShadow: "0px 1px 2px rgba(0,0,0,0.5)" }}
              >
                {label}
              </text>

              {hasGhost && (
                <text
                  x={w * UNIT - GAP - 6}
                  y={h * UNIT - GAP - 6}
                  textAnchor="end"
                  fill="rgba(255,255,255,0.3)"
                  fontSize="10"
                  fontWeight="bold"
                  className="pointer-events-none font-mono"
                >
                  {ghostLabel}
                </text>
              )}
            </g>
          );
        })}
      </svg>
    </div>
  );
});
