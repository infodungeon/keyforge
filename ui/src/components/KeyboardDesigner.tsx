import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { KeyboardDefinition } from "../types";
import { VisualBuilder } from "./VisualBuilder";
import { Save, Code, PenTool, LayoutTemplate } from "lucide-react";
import { Button } from "./ui/Button";
import { Input } from "./ui/Input";

interface Props {
    onSaveSuccess: () => void;
}

const DEFAULT_DEF: KeyboardDefinition = {
    meta: { name: "New Board", author: "Me", version: "1.0", notes: "", type: "ortho" },
    geometry: { keys: [], home_row: 1 },
    layouts: {}
};

export function KeyboardDesigner({ onSaveSuccess }: Props) {
    // Mode: 'visual' | 'code'
    const [mode, setMode] = useState<'visual' | 'code'>('visual');
    const [def, setDef] = useState<KeyboardDefinition>(DEFAULT_DEF);
    const [kleInput, setKleInput] = useState("");
    const [jsonError, setJsonError] = useState<string | null>(null);

    const handleParseKLE = async () => {
        if (!kleInput.trim()) return;
        try {
            const parsed = await invoke<KeyboardDefinition>("cmd_parse_kle", { json: kleInput });
            setDef({
                ...def,
                geometry: parsed.geometry,
                // Keep existing meta if set, else use parsed
                meta: { ...def.meta, notes: "Imported via KLE" }
            });
            setJsonError(null);
            setMode('visual'); // Switch back to visual after import
        } catch (e) {
            setJsonError(`Parse Failed: ${e}`);
        }
    };

    const handleSave = async () => {
        if (def.geometry.keys.length === 0) {
            alert("Cannot save empty keyboard.");
            return;
        }
        try {
            await invoke("cmd_save_keyboard", {
                filename: def.meta.name.toLowerCase().replace(/\s+/g, '_'),
                def
            });
            alert("Keyboard Saved!");
            onSaveSuccess();
        } catch (e) {
            alert(`Save failed: ${e}`);
        }
    };

    return (
        <div className="flex h-full w-full flex-col">

            {/* TOP BAR */}
            <div className="h-14 bg-slate-900 border-b border-slate-800 flex items-center px-6 justify-between shrink-0">
                <div className="flex items-center gap-4">
                    <h2 className="text-lg font-black text-white">Keyboard Designer</h2>

                    <div className="flex bg-slate-800 rounded p-0.5 border border-slate-700">
                        <button
                            onClick={() => setMode('visual')}
                            className={`flex items-center gap-2 px-3 py-1.5 rounded text-xs font-bold transition-all ${mode === 'visual' ? 'bg-blue-600 text-white shadow' : 'text-slate-400 hover:text-white'}`}
                        >
                            <LayoutTemplate size={14} /> Visual Editor
                        </button>
                        <button
                            onClick={() => setMode('code')}
                            className={`flex items-center gap-2 px-3 py-1.5 rounded text-xs font-bold transition-all ${mode === 'code' ? 'bg-purple-600 text-white shadow' : 'text-slate-400 hover:text-white'}`}
                        >
                            <Code size={14} /> KLE Import
                        </button>
                    </div>
                </div>

                <div className="flex items-center gap-4">
                    <Input
                        className="w-48 h-8"
                        placeholder="Keyboard Name"
                        value={def.meta.name}
                        onChange={e => setDef({ ...def, meta: { ...def.meta, name: e.target.value } })}
                    />
                    <Button variant="primary" size="sm" onClick={handleSave} icon={<Save size={14} />}>
                        Save
                    </Button>
                </div>
            </div>

            {/* MAIN CONTENT */}
            <div className="flex-1 flex overflow-hidden">

                {mode === 'visual' ? (
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
                                    Paste raw JSON from <a href="http://keyboard-layout-editor.com" target="_blank" className="text-blue-400 hover:underline">keyboard-layout-editor.com</a> to auto-generate geometry.
                                </p>

                                <textarea
                                    className="w-full h-64 bg-slate-950 border border-slate-800 rounded-lg p-4 text-xs font-mono text-slate-300 outline-none focus:border-purple-500 resize-none"
                                    placeholder='["Q", "W", "E", ...]'
                                    value={kleInput}
                                    onChange={e => setKleInput(e.target.value)}
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