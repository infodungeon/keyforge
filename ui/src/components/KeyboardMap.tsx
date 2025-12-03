import { KeyboardGeometry } from "../types";
import { keycodeService } from "../utils";

const UNIT = 54;
const GAP = 4;

function getKeyStyle(
  index: number,
  heatmap: number[] | undefined,
  maxHeat: number,
  isSelected: boolean,
  isEditing: boolean,
  isActive: boolean
) {
  // Priority 0: Physical Activation (Tester) - Bright Green
  if (isActive) {
    return { fill: "#22c55e", stroke: "#15803d", strokeWidth: 2, text: "#ffffff" };
  }

  // Base Fill
  let fill = "rgba(30, 41, 59, 1)";
  let stroke = "rgb(2, 6, 23)";
  let strokeWidth = 2;
  let text = "#94a3b8";

  // 1. Heatmap coloring
  if (heatmap && heatmap[index] && heatmap[index] > 0) {
    const val = heatmap[index];
    const intensity = Math.min(val / maxHeat, 1.0);
    const opacity = 0.1 + (intensity * 0.8);
    fill = `rgba(239, 68, 68, ${opacity})`;
    text = "#ffffff";
  } else {
    text = "#ffffff";
  }

  // 2. Selection Highlight
  if (isSelected) {
    stroke = "#3b82f6";
    strokeWidth = 3;
    if (!heatmap || !heatmap[index]) fill = "rgba(51, 65, 85, 1)";
  }

  // 3. Editing State
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
  // Mouse Events
  onKeyClick?: (index: number) => void; // Click (for selection)
  onKeyPointerDown?: (index: number) => void; // Press (for tester)
  onKeyPointerUp?: (index: number) => void;   // Release (for tester)

  activeKeyIds?: Set<string>;
}

export function KeyboardMap({
  geometry, layoutString, heatmap, className = "",
  selectedKeyIndex, isEditing = false,
  onKeyClick, onKeyPointerDown, onKeyPointerUp,
  activeKeyIds
}: KeyboardMapProps) {

  if (!geometry || !geometry.keys) return (
    <div className="flex flex-col items-center justify-center h-full text-slate-600 font-mono text-xs gap-2">
      <div className="animate-pulse bg-slate-800/50 h-32 w-64 rounded-lg border border-slate-700/50"></div>
      <div>NO GEOMETRY LOADED</div>
    </div>
  );

  const maxX = Math.max(...geometry.keys.map((k) => k.x + (k.w || 1)));
  const maxY = Math.max(...geometry.keys.map((k) => k.y + (k.h || 1)));
  const maxHeat = heatmap ? Math.max(...heatmap) : 0.12;
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
            index, heatmap, maxHeat,
            selectedKeyIndex === index,
            selectedKeyIndex === index && isEditing,
            isActive
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

          // 3D Depressed Effect Logic
          // Normal: Translate(0,0)
          // Pressed (Active): Translate(0, 2px) -> Looks like it went down
          // Hover: Translate(0, -1px) -> Looks slightly raised/interactive
          const yOffset = isActive ? 2 : 0;
          const transform = `translate(${key.x * UNIT}px, ${key.y * UNIT + yOffset}px)`;

          return (
            <g
              key={index}
              style={{ transform, transition: 'transform 50ms ease-out' }}
              onPointerDown={(e) => {
                e.preventDefault(); // Prevent focus theft
                onKeyPointerDown && onKeyPointerDown(index);
              }}
              onPointerUp={(e) => {
                e.preventDefault();
                onKeyPointerUp && onKeyPointerUp(index);
              }}
              onPointerLeave={() => {
                // If mouse leaves key while pressed, cancel the press
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
                fill={style.fill}
                stroke={style.stroke}
                strokeWidth={style.strokeWidth}
                className="transition-colors duration-75"
              // Shadow effect via filter can be added here if desired
              />
              <text
                x={(w * UNIT - GAP) / 2}
                y={(h * UNIT - GAP) / 2 + 7}
                textAnchor="middle"
                alignmentBaseline="middle"
                fill={style.text}
                fontSize={label.length > 2 ? 12 : 18}
                className="font-bold pointer-events-none font-mono"
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