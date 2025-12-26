import { useState, useRef, useEffect } from "react";
import { KeyNode, KeyboardGeometry } from "../types";
import {
  Plus,
  Trash2,
  Copy,
  Grid,
  Move,
  AlignStartVertical,
  AlignCenterHorizontal,
  MousePointer2,
} from "lucide-react";
import { Button } from "./ui/Button";
import { Label } from "./ui/Label";
import { Input } from "./ui/Input";

interface Props {
  geometry: KeyboardGeometry;
  onChange: (geo: KeyboardGeometry) => void;
}

const UNIT = 54; // Visual Pixel scale per Key Unit (1u)
const SNAP = 0.25; // Snap to quarter units

export function VisualBuilder({ geometry, onChange }: Props) {
  // State: Multi-selection support
  const [selectedIndices, setSelectedIndices] = useState<Set<number>>(
    new Set(),
  );
  const [isDragging, setIsDragging] = useState(false);

  // Drag State: Tracks initial mouse pos and initial key positions for ALL selected keys
  const dragStartRef = useRef<{
    startX: number;
    startY: number;
    initialKeys: Map<number, { x: number; y: number }>;
  } | null>(null);

  const svgRef = useRef<SVGSVGElement>(null);

  // --- Selection Logic ---

  const handleKeyClick = (e: React.MouseEvent, idx: number) => {
    e.stopPropagation();
    const newSet = new Set(e.shiftKey ? selectedIndices : []);

    if (newSet.has(idx)) {
      newSet.delete(idx);
    } else {
      newSet.add(idx);
    }
    setSelectedIndices(newSet);
  };

  const handleBackgroundClick = () => {
    setSelectedIndices(new Set());
  };

  // --- Manipulation Logic ---

  const updateKeys = (updates: Map<number, Partial<KeyNode>>) => {
    const newKeys = [...geometry.keys];
    updates.forEach((u, idx) => {
      if (idx < newKeys.length) {
        newKeys[idx] = { ...newKeys[idx], ...u };
      }
    });
    onChange({ ...geometry, keys: newKeys });
  };

  const handleMouseDown = (e: React.MouseEvent, idx: number) => {
    e.stopPropagation();

    // If clicking an unselected key without shift, select only it
    let newSelection = new Set(selectedIndices);
    if (!e.shiftKey && !newSelection.has(idx)) {
      newSelection = new Set([idx]);
      setSelectedIndices(newSelection);
    } else if (e.shiftKey && !newSelection.has(idx)) {
      newSelection.add(idx);
      setSelectedIndices(newSelection);
    }

    setIsDragging(true);

    // Snapshot positions of ALL selected keys
    const initialMap = new Map();
    newSelection.forEach((i) => {
      if (geometry.keys[i]) {
        initialMap.set(i, { x: geometry.keys[i].x, y: geometry.keys[i].y });
      }
    });

    dragStartRef.current = {
      startX: e.clientX,
      startY: e.clientY,
      initialKeys: initialMap,
    };
  };

  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (!isDragging || !dragStartRef.current) return;

      const dx_px = e.clientX - dragStartRef.current.startX;
      const dy_px = e.clientY - dragStartRef.current.startY;

      const dx_u = dx_px / UNIT;
      const dy_u = dy_px / UNIT;

      const updates = new Map();

      dragStartRef.current.initialKeys.forEach((pos, idx) => {
        let newX = pos.x + dx_u;
        let newY = pos.y + dy_u;

        // Snap
        newX = Math.round(newX / SNAP) * SNAP;
        newY = Math.round(newY / SNAP) * SNAP;

        updates.set(idx, { x: newX, y: newY });
      });

      updateKeys(updates);
    };

    const handleMouseUp = () => {
      setIsDragging(false);
      dragStartRef.current = null;
    };

    if (isDragging) {
      window.addEventListener("mousemove", handleMouseMove);
      window.addEventListener("mouseup", handleMouseUp);
    }
    return () => {
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
    };
  }, [isDragging, geometry]);

  // --- Tools ---

  const addKey = () => {
    const newKey: KeyNode = {
      id: `k${geometry.keys.length}`,
      x: 0,
      y: 0,
      w: 1,
      h: 1,
      hand: 0,
      finger: 1,
      row: 0,
      col: 0,
      is_stretch: false,
    };

    // Place near last selected or last key
    const refIdx =
      Array.from(selectedIndices).pop() ?? geometry.keys.length - 1;
    if (refIdx >= 0 && geometry.keys[refIdx]) {
      const last = geometry.keys[refIdx];
      newKey.x = last.x + (last.w || 1);
      newKey.y = last.y;
      newKey.hand = last.hand;
    }

    const newKeys = [...geometry.keys, newKey];
    onChange({ ...geometry, keys: newKeys });
    setSelectedIndices(new Set([newKeys.length - 1]));
  };

  const deleteSelected = () => {
    if (selectedIndices.size === 0) return;
    const newKeys = geometry.keys.filter((_, i) => !selectedIndices.has(i));
    onChange({ ...geometry, keys: newKeys });
    setSelectedIndices(new Set());
  };

  const alignX = () => {
    if (selectedIndices.size < 2) return;
    const firstIdx = selectedIndices.values().next().value;
    const targetX = geometry.keys[firstIdx].x;

    const updates = new Map();
    selectedIndices.forEach((idx) => updates.set(idx, { x: targetX }));
    updateKeys(updates);
  };

  const alignY = () => {
    if (selectedIndices.size < 2) return;
    const firstIdx = selectedIndices.values().next().value;
    const targetY = geometry.keys[firstIdx].y;

    const updates = new Map();
    selectedIndices.forEach((idx) => updates.set(idx, { y: targetY }));
    updateKeys(updates);
  };

  // --- Render Helpers ---

  // Calculate bounding box for viewbox
  const maxX = Math.max(15, ...geometry.keys.map((k) => k.x + (k.w || 1)));
  const maxY = Math.max(5, ...geometry.keys.map((k) => k.y + (k.h || 1)));

  // Properties Panel Helper
  const singleSelectedKey =
    selectedIndices.size === 1
      ? geometry.keys[Array.from(selectedIndices)[0]]
      : null;

  return (
    <div className="flex h-full w-full">
      {/* CANVAS */}
      <div
        className="flex-1 bg-[#0B0F19] relative overflow-hidden flex items-center justify-center"
        onClick={handleBackgroundClick}
      >
        {/* Grid Background */}
        <div
          className="absolute inset-0 opacity-10 pointer-events-none"
          style={{
            backgroundImage: `radial-gradient(#475569 1px, transparent 1px)`,
            backgroundSize: `${UNIT / 2}px ${UNIT / 2}px`,
          }}
        />

        <svg
          ref={svgRef}
          width="90%"
          height="90%"
          viewBox={`-1 -1 ${maxX + 2} ${maxY + 2}`}
          className="overflow-visible"
        >
          {geometry.keys.map((k, i) => {
            const isSel = selectedIndices.has(i);
            const fill = k.hand === 0 ? "#1e293b" : "#0f172a";
            const stroke = isSel ? "#3b82f6" : "#334155";
            const width = k.w || 1;
            const height = k.h || 1;

            return (
              <g
                key={i}
                transform={`translate(${k.x}, ${k.y})`}
                onMouseDown={(e) => handleMouseDown(e, i)}
                onClick={(e) => handleKeyClick(e, i)}
                className="cursor-grab active:cursor-grabbing"
              >
                <rect
                  width={width - 0.05}
                  height={height - 0.05}
                  rx={0.15}
                  fill={fill}
                  stroke={stroke}
                  strokeWidth={isSel ? 0.08 : 0.02}
                  vectorEffect="non-scaling-stroke"
                  className="transition-colors"
                />
                <text
                  x={width / 2}
                  y={height / 2}
                  fontSize={0.25}
                  fill={isSel ? "white" : "#64748b"}
                  textAnchor="middle"
                  alignmentBaseline="middle"
                  pointerEvents="none"
                  className="font-mono select-none"
                >
                  {i}
                </text>
                <circle
                  cx={width - 0.2}
                  cy={height - 0.2}
                  r={0.08}
                  fill={
                    ["#64748b", "#22c55e", "#3b82f6", "#a855f7", "#ec4899"][
                      k.finger % 5
                    ]
                  }
                />
              </g>
            );
          })}
        </svg>

        {/* Floating Toolbar */}
        <div className="absolute top-4 left-1/2 -translate-x-1/2 flex gap-2 bg-slate-900/90 border border-slate-800 p-2 rounded-xl shadow-xl backdrop-blur">
          <Button
            size="sm"
            variant="secondary"
            onClick={addKey}
            icon={<Plus size={14} />}
          >
            Add
          </Button>
          <div className="w-px bg-slate-700 mx-1" />
          <Button
            size="sm"
            variant="ghost"
            onClick={alignX}
            disabled={selectedIndices.size < 2}
            title="Align Vertical (X)"
          >
            <AlignStartVertical size={14} />
          </Button>
          <Button
            size="sm"
            variant="ghost"
            onClick={alignY}
            disabled={selectedIndices.size < 2}
            title="Align Horizontal (Y)"
          >
            <AlignCenterHorizontal size={14} />
          </Button>
          <div className="w-px bg-slate-700 mx-1" />
          <Button
            size="sm"
            variant="danger"
            onClick={deleteSelected}
            disabled={selectedIndices.size === 0}
            icon={<Trash2 size={14} />}
          >
            Del
          </Button>
        </div>

        <div className="absolute bottom-4 left-4 text-[10px] text-slate-500 font-mono">
          {selectedIndices.size} keys selected. Shift+Click to multi-select.
          Drag to move.
        </div>
      </div>

      {/* INSPECTOR */}
      <div className="w-80 bg-slate-900 border-l border-slate-800 flex flex-col shrink-0">
        <div className="p-4 border-b border-slate-800 bg-slate-950/30">
          <h3 className="text-xs font-bold text-slate-400 uppercase flex items-center gap-2">
            <Grid size={14} /> Properties
          </h3>
        </div>

        {singleSelectedKey ? (
          <div className="p-4 space-y-6 overflow-y-auto custom-scrollbar">
            {/* Position */}
            <div className="space-y-3">
              <div className="flex items-center gap-2 text-slate-200 text-xs font-bold border-b border-slate-800 pb-1">
                <Move size={12} /> Geometry (Units)
              </div>
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <Label>X</Label>
                  <Input
                    type="number"
                    step="0.25"
                    value={singleSelectedKey.x}
                    onChange={(e) =>
                      updateKeys(
                        new Map([
                          [
                            Array.from(selectedIndices)[0],
                            { x: parseFloat(e.target.value) },
                          ],
                        ]),
                      )
                    }
                  />
                </div>
                <div>
                  <Label>Y</Label>
                  <Input
                    type="number"
                    step="0.25"
                    value={singleSelectedKey.y}
                    onChange={(e) =>
                      updateKeys(
                        new Map([
                          [
                            Array.from(selectedIndices)[0],
                            { y: parseFloat(e.target.value) },
                          ],
                        ]),
                      )
                    }
                  />
                </div>
                <div>
                  <Label>Width</Label>
                  <Input
                    type="number"
                    step="0.25"
                    value={singleSelectedKey.w || 1}
                    onChange={(e) =>
                      updateKeys(
                        new Map([
                          [
                            Array.from(selectedIndices)[0],
                            { w: parseFloat(e.target.value) },
                          ],
                        ]),
                      )
                    }
                  />
                </div>
                <div>
                  <Label>Height</Label>
                  <Input
                    type="number"
                    step="0.25"
                    value={singleSelectedKey.h || 1}
                    onChange={(e) =>
                      updateKeys(
                        new Map([
                          [
                            Array.from(selectedIndices)[0],
                            { h: parseFloat(e.target.value) },
                          ],
                        ]),
                      )
                    }
                  />
                </div>
              </div>
            </div>

            {/* Physics */}
            <div className="space-y-3">
              <div className="flex items-center gap-2 text-slate-200 text-xs font-bold border-b border-slate-800 pb-1">
                Physics
              </div>
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <Label>Hand</Label>
                  <select
                    className="w-full bg-slate-950 border border-slate-800 rounded px-2 py-2 text-xs text-slate-200"
                    value={singleSelectedKey.hand}
                    onChange={(e) =>
                      updateKeys(
                        new Map([
                          [
                            Array.from(selectedIndices)[0],
                            { hand: parseInt(e.target.value) },
                          ],
                        ]),
                      )
                    }
                  >
                    <option value={0}>Left (0)</option>
                    <option value={1}>Right (1)</option>
                  </select>
                </div>
                <div>
                  <Label>Finger</Label>
                  <select
                    className="w-full bg-slate-950 border border-slate-800 rounded px-2 py-2 text-xs text-slate-200"
                    value={singleSelectedKey.finger}
                    onChange={(e) =>
                      updateKeys(
                        new Map([
                          [
                            Array.from(selectedIndices)[0],
                            { finger: parseInt(e.target.value) },
                          ],
                        ]),
                      )
                    }
                  >
                    <option value={0}>Thumb (0)</option>
                    <option value={1}>Index (1)</option>
                    <option value={2}>Middle (2)</option>
                    <option value={3}>Ring (3)</option>
                    <option value={4}>Pinky (4)</option>
                  </select>
                </div>
              </div>
              <div className="flex items-center gap-2 pt-2">
                <input
                  type="checkbox"
                  checked={singleSelectedKey.is_stretch || false}
                  onChange={(e) =>
                    updateKeys(
                      new Map([
                        [
                          Array.from(selectedIndices)[0],
                          { is_stretch: e.target.checked },
                        ],
                      ]),
                    )
                  }
                  className="accent-purple-500"
                />
                <span className="text-xs text-slate-400">
                  Lateral Stretch Column
                </span>
              </div>
            </div>
          </div>
        ) : (
          <div className="p-8 text-center text-slate-600 text-xs italic flex flex-col items-center gap-2">
            <MousePointer2 size={24} className="opacity-50" />
            {selectedIndices.size > 1 ? (
              <span>
                {selectedIndices.size} items selected.
                <br />
                Bulk editing not yet supported.
              </span>
            ) : (
              <span>Select a key to edit properties.</span>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
