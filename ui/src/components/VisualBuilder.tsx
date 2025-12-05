import { useState, useRef, useEffect } from "react";
import { KeyNode, KeyboardGeometry } from "../types";
import { Plus, Trash2, Copy, Grid, Move } from "lucide-react";
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
    const [selectedIdx, setSelectedIdx] = useState<number | null>(null);
    const [isDragging, setIsDragging] = useState(false);
    const dragStartRef = useRef<{ x: number, y: number, keyX: number, keyY: number } | null>(null);
    const svgRef = useRef<SVGSVGElement>(null);

    // --- Actions ---

    const addKey = () => {
        const newKey: KeyNode = {
            id: `k${geometry.keys.length}`,
            x: 0, y: 0, w: 1, h: 1,
            hand: 0, finger: 1, row: 0, col: 0,
            is_stretch: false
        };
        // Place next to the last key if exists
        if (geometry.keys.length > 0) {
            const last = geometry.keys[geometry.keys.length - 1];
            newKey.x = last.x + (last.w || 1);
            newKey.y = last.y;
            newKey.row = last.row;
            newKey.col = last.col + 1;
            newKey.hand = last.hand;
        }

        const newKeys = [...geometry.keys, newKey];
        onChange({ ...geometry, keys: newKeys });
        setSelectedIdx(newKeys.length - 1);
    };

    const updateKey = (idx: number, updates: Partial<KeyNode>) => {
        const newKeys = [...geometry.keys];
        newKeys[idx] = { ...newKeys[idx], ...updates };
        onChange({ ...geometry, keys: newKeys });
    };

    const deleteKey = () => {
        if (selectedIdx === null) return;
        const newKeys = geometry.keys.filter((_, i) => i !== selectedIdx);
        onChange({ ...geometry, keys: newKeys });
        setSelectedIdx(null);
    };

    const duplicateKey = () => {
        if (selectedIdx === null) return;
        const source = geometry.keys[selectedIdx];
        const newKey = { ...source, id: `k${geometry.keys.length}`, x: source.x + 0.25, y: source.y + 0.25 };
        const newKeys = [...geometry.keys, newKey];
        onChange({ ...geometry, keys: newKeys });
        setSelectedIdx(newKeys.length - 1);
    };

    // --- Drag & Drop Logic ---

    const handleMouseDown = (e: React.MouseEvent, idx: number) => {
        e.stopPropagation();
        setSelectedIdx(idx);
        setIsDragging(true);
        dragStartRef.current = {
            x: e.clientX,
            y: e.clientY,
            keyX: geometry.keys[idx].x,
            keyY: geometry.keys[idx].y
        };
    };

    useEffect(() => {
        const handleMouseMove = (e: MouseEvent) => {
            if (!isDragging || selectedIdx === null || !dragStartRef.current) return;

            const dx_px = e.clientX - dragStartRef.current.x;
            const dy_px = e.clientY - dragStartRef.current.y;

            // Convert pixels to Key Units
            const dx_u = dx_px / UNIT;
            const dy_u = dy_px / UNIT;

            let newX = dragStartRef.current.keyX + dx_u;
            let newY = dragStartRef.current.keyY + dy_u;

            // Snap
            newX = Math.round(newX / SNAP) * SNAP;
            newY = Math.round(newY / SNAP) * SNAP;

            updateKey(selectedIdx, { x: newX, y: newY });
        };

        const handleMouseUp = () => {
            setIsDragging(false);
            dragStartRef.current = null;
        };

        if (isDragging) {
            window.addEventListener('mousemove', handleMouseMove);
            window.addEventListener('mouseup', handleMouseUp);
        }
        return () => {
            window.removeEventListener('mousemove', handleMouseMove);
            window.removeEventListener('mouseup', handleMouseUp);
        };
    }, [isDragging, selectedIdx, geometry]);

    // --- Render Helpers ---

    const selKey = selectedIdx !== null ? geometry.keys[selectedIdx] : null;

    // Viewbox calculation
    const maxX = Math.max(15, ...geometry.keys.map(k => k.x + (k.w || 1)));
    const maxY = Math.max(5, ...geometry.keys.map(k => k.y + (k.h || 1)));

    return (
        <div className="flex h-full w-full">
            {/* CANVAS */}
            <div
                className="flex-1 bg-[#0B0F19] relative overflow-hidden flex items-center justify-center"
                onClick={() => setSelectedIdx(null)}
            >
                {/* Grid Background */}
                <div
                    className="absolute inset-0 opacity-10 pointer-events-none"
                    style={{
                        backgroundImage: `radial-gradient(#475569 1px, transparent 1px)`,
                        backgroundSize: `${UNIT / 2}px ${UNIT / 2}px`
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
                        const isSel = selectedIdx === i;
                        const fill = k.hand === 0 ? "#1e293b" : "#0f172a"; // L/R distinct slate
                        const stroke = isSel ? "#3b82f6" : "#334155";
                        const width = k.w || 1;
                        const height = k.h || 1;

                        return (
                            <g
                                key={i}
                                transform={`translate(${k.x}, ${k.y})`}
                                onMouseDown={(e) => handleMouseDown(e, i)}
                                className="cursor-grab active:cursor-grabbing"
                            >
                                <rect
                                    width={width - 0.05}
                                    height={height - 0.05}
                                    rx={0.15}
                                    fill={fill}
                                    stroke={stroke}
                                    strokeWidth={isSel ? 0.05 : 0.02}
                                    vectorEffect="non-scaling-stroke"
                                    className="transition-colors"
                                />
                                {/* Label: Index + Hand/Finger */}
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
                                {/* Finger Dot */}
                                <circle
                                    cx={width - 0.2}
                                    cy={height - 0.2}
                                    r={0.08}
                                    fill={["#64748b", "#22c55e", "#3b82f6", "#a855f7", "#ec4899"][k.finger % 5]}
                                />
                            </g>
                        );
                    })}
                </svg>

                {/* Floating Toolbar */}
                <div className="absolute top-4 left-1/2 -translate-x-1/2 flex gap-2 bg-slate-900/90 border border-slate-800 p-2 rounded-xl shadow-xl backdrop-blur">
                    <Button size="sm" variant="secondary" onClick={addKey} icon={<Plus size={14} />}>Add</Button>
                    <Button size="sm" variant="secondary" onClick={duplicateKey} disabled={selectedIdx === null} icon={<Copy size={14} />}>Dup</Button>
                    <Button size="sm" variant="danger" onClick={deleteKey} disabled={selectedIdx === null} icon={<Trash2 size={14} />}>Del</Button>
                </div>
            </div>

            {/* INSPECTOR */}
            <div className="w-80 bg-slate-900 border-l border-slate-800 flex flex-col shrink-0">
                <div className="p-4 border-b border-slate-800 bg-slate-950/30">
                    <h3 className="text-xs font-bold text-slate-400 uppercase flex items-center gap-2">
                        <Grid size={14} /> Properties
                    </h3>
                </div>

                {selKey ? (
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
                                        type="number" step="0.25"
                                        value={selKey.x}
                                        onChange={e => updateKey(selectedIdx!, { x: parseFloat(e.target.value) })}
                                    />
                                </div>
                                <div>
                                    <Label>Y</Label>
                                    <Input
                                        type="number" step="0.25"
                                        value={selKey.y}
                                        onChange={e => updateKey(selectedIdx!, { y: parseFloat(e.target.value) })}
                                    />
                                </div>
                                <div>
                                    <Label>Width</Label>
                                    <Input
                                        type="number" step="0.25"
                                        value={selKey.w || 1}
                                        onChange={e => updateKey(selectedIdx!, { w: parseFloat(e.target.value) })}
                                    />
                                </div>
                                <div>
                                    <Label>Height</Label>
                                    <Input
                                        type="number" step="0.25"
                                        value={selKey.h || 1}
                                        onChange={e => updateKey(selectedIdx!, { h: parseFloat(e.target.value) })}
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
                                        className="w-full bg-slate-950 border border-slate-800 rounded px-2 py-2 text-xs"
                                        value={selKey.hand}
                                        onChange={e => updateKey(selectedIdx!, { hand: parseInt(e.target.value) })}
                                    >
                                        <option value={0}>Left (0)</option>
                                        <option value={1}>Right (1)</option>
                                    </select>
                                </div>
                                <div>
                                    <Label>Finger</Label>
                                    <select
                                        className="w-full bg-slate-950 border border-slate-800 rounded px-2 py-2 text-xs"
                                        value={selKey.finger}
                                        onChange={e => updateKey(selectedIdx!, { finger: parseInt(e.target.value) })}
                                    >
                                        <option value={0}>Thumb (0)</option>
                                        <option value={1}>Index (1)</option>
                                        <option value={2}>Middle (2)</option>
                                        <option value={3}>Ring (3)</option>
                                        <option value={4}>Pinky (4)</option>
                                    </select>
                                </div>
                                <div>
                                    <Label>Row</Label>
                                    <Input
                                        type="number"
                                        value={selKey.row}
                                        onChange={e => updateKey(selectedIdx!, { row: parseInt(e.target.value) })}
                                    />
                                </div>
                                <div>
                                    <Label>Col</Label>
                                    <Input
                                        type="number"
                                        value={selKey.col}
                                        onChange={e => updateKey(selectedIdx!, { col: parseInt(e.target.value) })}
                                    />
                                </div>
                            </div>
                            <div className="flex items-center gap-2 pt-2">
                                <input
                                    type="checkbox"
                                    checked={selKey.is_stretch || false}
                                    onChange={e => updateKey(selectedIdx!, { is_stretch: e.target.checked })}
                                    className="accent-purple-500"
                                />
                                <span className="text-xs text-slate-400">Lateral Stretch Column</span>
                            </div>
                        </div>

                    </div>
                ) : (
                    <div className="p-8 text-center text-slate-600 text-xs italic">
                        Select a key to edit properties.
                    </div>
                )}
            </div>
        </div>
    );
}