import { useState, useEffect } from "react";
import { KeyboardDefinition } from "../types";
import { VisualBuilder } from "./VisualBuilder";
import { Save, Code, PenTool, LayoutTemplate, Undo, Redo } from "lucide-react";
import { Button } from "./ui/Button";
import { Input } from "./ui/Input";
import { useUndo } from "../hooks/useUndo";
import { useBackend } from "../context/BackendContext";

interface Props {
  onSaveSuccess: () => void;
}

const DEFAULT_DEF: KeyboardDefinition = {
  meta: {
    name: "New Board",
    author: "Me",
    version: "1.0",
    notes: "",
    type: "ortho",
  },
  geometry: {
    keys: [],
    prime_slots: [],
    med_slots: [],
    low_slots: [],
    home_row: 1,
  },
  layouts: {},
};

export function KeyboardDesigner({ onSaveSuccess }: Props) {
  const backend = useBackend();
  const [mode, setMode] = useState<"visual" | "code">("visual");
  const [kleInput, setKleInput] = useState("");
  const [jsonError, setJsonError] = useState<string | null>(null);

  // UX: Undo/Redo Hook
  const {
    state: def,
    set: setDef,
    undo,
    redo,
    canUndo,
    canRedo,
  } = useUndo<KeyboardDefinition>(DEFAULT_DEF);

  // Keyboard Shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "z") {
        e.preventDefault();
        if (e.shiftKey) redo();
        else undo();
      }
      if ((e.ctrlKey || e.metaKey) && e.key === "y") {
        e.preventDefault();
        redo();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [undo, redo]);

  const handleParseKLE = async () => {
    if (!kleInput.trim()) return;
    try {
      const parsed = await backend.parseKle(kleInput);
      setDef({
        ...def,
        geometry: parsed.geometry,
        meta: { ...def.meta, notes: "Imported via KLE" },
      });
      setJsonError(null);
      setMode("visual");
    } catch (e) {
      setJsonError(`Parse Failed: ${e}`);
    }
  };

  const handleSave = async () => {
    // Validation
    if (def.geometry.keys.length === 0) {
      alert("Cannot save empty keyboard.");
      return;
    }
    if (!def.meta.name.trim()) {
      alert("Keyboard name is required.");
      return;
    }
    // Check for duplicate key IDs
    const ids = new Set();
    for (const k of def.geometry.keys) {
      if (ids.has(k.label)) {
        alert(`Duplicate Key ID found: ${k.label}`);
        return;
      }
      ids.add(k.label);
    }
    if (def.geometry.keys.length === 0) {
      alert("Cannot save empty keyboard.");
      return;
    }
    try {
      await backend.saveKeyboard(
        def.meta.name.toLowerCase().replace(/\s+/g, "_"),
        def,
      );
      alert("Keyboard Saved!");
      onSaveSuccess();
    } catch (e) {
      alert(`Save failed: ${e}`);
    }
  };

  return (
    <div className="flex h-full w-full flex-col">
      <div className="h-14 bg-slate-900 border-b border-slate-800 flex items-center px-6 justify-between shrink-0">
        <div className="flex items-center gap-4">
          <h2 className="text-lg font-black text-white">Keyboard Designer</h2>

          <div className="flex bg-slate-800 rounded p-0.5 border border-slate-700">
            <button
              onClick={() => setMode("visual")}
              className={`flex items-center gap-2 px-3 py-1.5 rounded text-xs font-bold transition-all ${mode === "visual" ? "bg-blue-600 text-white shadow" : "text-slate-400 hover:text-white"}`}
            >
              <LayoutTemplate size={14} /> Visual
            </button>
            <button
              onClick={() => setMode("code")}
              className={`flex items-center gap-2 px-3 py-1.5 rounded text-xs font-bold transition-all ${mode === "code" ? "bg-purple-600 text-white shadow" : "text-slate-400 hover:text-white"}`}
            >
              <Code size={14} /> Import
            </button>
          </div>

          <div className="h-6 w-px bg-slate-800 mx-2" />

          <div className="flex gap-1">
            <Button
              variant="ghost"
              size="sm"
              onClick={undo}
              disabled={!canUndo}
              title="Undo (Ctrl+Z)"
            >
              <Undo size={14} />
            </Button>
            <Button
              variant="ghost"
              size="sm"
              onClick={redo}
              disabled={!canRedo}
              title="Redo (Ctrl+Y)"
            >
              <Redo size={14} />
            </Button>
          </div>
        </div>

        <div className="flex items-center gap-4">
          <Input
            className="w-48 h-8"
            placeholder="Keyboard Name"
            value={def.meta.name}
            onChange={(e) =>
              setDef({ ...def, meta: { ...def.meta, name: e.target.value } })
            }
          />
          <Button
            variant="primary"
            size="sm"
            onClick={handleSave}
            icon={<Save size={14} />}
          >
            Save
          </Button>
        </div>
      </div>

      <div className="flex-1 flex overflow-hidden">
        {mode === "visual" ? (
          <VisualBuilder
            geometry={def.geometry}
            onChange={(geo) => setDef({ ...def, geometry: geo })}
          />
        ) : (
          <div className="flex-1 bg-[#0B0F19] p-12 flex flex-col items-center">
            <div className="w-full max-w-2xl space-y-6">
              <div className="bg-slate-900 border border-slate-800 rounded-xl p-6">
                <h3 className="text-sm font-bold text-white mb-2 flex items-center gap-2">
                  <PenTool size={16} /> Import from Keyboard Layout Editor
                </h3>
                <p className="text-xs text-slate-500 mb-4">
                  Paste raw JSON from keyboard-layout-editor.com
                </p>
                <textarea
                  className="w-full h-64 bg-slate-950 border border-slate-800 rounded-lg p-4 text-xs font-mono text-slate-300 outline-none focus:border-purple-500 resize-none"
                  placeholder='["Q", "W", "E", ...]'
                  value={kleInput}
                  onChange={(e) => setKleInput(e.target.value)}
                />
                {jsonError && (
                  <div className="mt-4 p-3 bg-red-900/20 border border-red-900/50 rounded text-red-400 text-xs font-mono">
                    {jsonError}
                  </div>
                )}
                <div className="mt-4 flex justify-end">
                  <Button variant="secondary" onClick={handleParseKLE}>
                    Parse & Load
                  </Button>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
