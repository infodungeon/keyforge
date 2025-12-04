import { useEffect, useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Keyboard, RotateCcw } from "lucide-react";
import { Button } from "./ui/Button";
import { KeyboardMap } from "./KeyboardMap";
import { KeyboardGeometry } from "../types";
import { useToast } from "../context/ToastContext";

export function KeyTester() {
    const { addToast } = useToast();
    const [history, setHistory] = useState<{ key: string, code: string }[]>([]);
    const [activeKeys, setActiveKeys] = useState<Set<string>>(new Set());
    const [geometry, setGeometry] = useState<KeyboardGeometry | null>(null);
    const [error, setError] = useState<string | null>(null);

    // Ref to capture focus
    const containerRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        invoke<KeyboardGeometry>("cmd_get_keyboard_geometry", { name: "ansi_104" })
            .then(setGeometry)
            .catch(e => {
                console.error(e);
                setError("Failed to load ANSI Layout. Please ensure standard keyboards are loaded.");
                addToast('error', "Failed to load ANSI layout for tester.");
            });

        // Focus immediately so user can type
        if (containerRef.current) containerRef.current.focus();
    }, [addToast]);

    const handleKeyDown = (e: KeyboardEvent) => {
        e.preventDefault();
        // e.code is the physical key code (e.g. "KeyA", "Space") which matches our ANSI IDs
        setActiveKeys(prev => {
            const next = new Set(prev);
            next.add(e.code);
            return next;
        });
        if (!e.repeat) {
            setHistory(prev => [{ key: e.key, code: e.code }, ...prev].slice(0, 50));
        }
    };

    const handleKeyUp = (e: KeyboardEvent) => {
        e.preventDefault();
        setActiveKeys(prev => {
            const next = new Set(prev);
            next.delete(e.code);
            return next;
        });
    };

    // Mouse Interaction Handlers
    const handleMouseDown = (index: number) => {
        if (!geometry) return;
        const keyId = geometry.keys[index].id || "";
        setActiveKeys(prev => new Set(prev).add(keyId));
        setHistory(prev => [{ key: "Click", code: keyId }, ...prev].slice(0, 50));
    };

    const handleMouseUp = (index: number) => {
        if (!geometry) return;
        const keyId = geometry.keys[index].id || "";
        setActiveKeys(prev => {
            const next = new Set(prev);
            next.delete(keyId);
            return next;
        });
    };

    return (
        <div
            ref={containerRef}
            className="flex-1 flex flex-col bg-[#0B0F19] p-8 overflow-hidden outline-none"
            tabIndex={0} // Make div focusable
            onKeyDown={handleKeyDown as any}
            onKeyUp={handleKeyUp as any}
        >
            {/* Header */}
            <div className="flex items-center justify-between mb-4 border-b border-slate-800 pb-4 shrink-0">
                <div className="flex items-center gap-3">
                    <Keyboard size={24} className="text-green-500" />
                    <h2 className="text-xl font-black text-white">Input Tester</h2>
                </div>
                <div className="flex items-center gap-4">
                    <span className="text-[10px] text-slate-500 font-mono">
                        Press keys to verify codes
                    </span>
                    <Button variant="secondary" size="sm" onClick={() => setHistory([])} icon={<RotateCcw size={14} />}>
                        Clear Log
                    </Button>
                </div>
            </div>

            {/* Visualizer */}
            <div className="flex-1 flex items-center justify-center overflow-hidden pb-4 relative">
                {error ? (
                    <div className="text-red-400 font-mono text-sm bg-red-900/20 p-4 rounded border border-red-900/50">
                        {error}
                    </div>
                ) : geometry ? (
                    <KeyboardMap
                        geometry={geometry}
                        layoutString="" // No labels needed for physical tester usually, or could use IDs
                        activeKeyIds={activeKeys}
                        // Wire up mouse events for momentary press
                        onKeyPointerDown={handleMouseDown}
                        onKeyPointerUp={handleMouseUp}
                        className="w-full h-full"
                    />
                ) : (
                    <div className="text-slate-500 animate-pulse">Loading ANSI Layout...</div>
                )}
            </div>

            {/* History Log */}
            <div className="h-24 shrink-0 border-t border-slate-800 pt-4">
                <div className="text-[10px] font-bold text-slate-500 uppercase mb-2">Event Log</div>
                <div className="flex gap-2 overflow-x-auto pb-2 scrollbar-thin scrollbar-thumb-slate-800">
                    {history.map((h, i) => (
                        <div key={i} className="flex flex-col items-center shrink-0 min-w-[3rem]">
                            <div className={`
                                px-3 py-1.5 rounded-lg border text-xs font-mono font-bold mb-1 transition-all whitespace-nowrap
                                ${i === 0 ? "bg-slate-700 border-slate-500 text-white" : "bg-slate-900/50 border-slate-800 text-slate-500"}
                            `}>
                                {h.code}
                            </div>
                            <span className="text-[9px] text-slate-600">{h.key}</span>
                        </div>
                    ))}
                </div>
            </div>
        </div>
    );
}