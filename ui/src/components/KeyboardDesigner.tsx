import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { KeyboardDefinition } from "../types";
import { KeyboardMap } from "./KeyboardMap";
import { Save, PenTool } from "lucide-react"; // FIXED: Removed LayoutTemplate
import { Button } from "./ui/Button";
import { Select } from "./ui/Select";
import { Label } from "./ui/Label";
import { Input } from "./ui/Input";

interface Props {
    onSaveSuccess: () => void;
}

const TEMPLATES = [
    { label: "Empty Canvas", value: "" },
    { label: "Ortho 30 (Planck-ish)", value: `[["Q","W","E","R","T","Y","U","I","O","P"],["A","S","D","F","G","H","J","K","L",";"],["Z","X","C","V","B","N","M",",",".","/"]]` },
    { label: "Split 36 (Corne-ish)", value: `[["Q","W","E","R","T","Y","U","I","O","P"],["A","S","D","F","G","H","J","K","L",";"],["Z","X","C","V","B","N","M",",",".","/"],[{y:0.5},"L1","L2","L3",{x:2},"R1","R2","R3"]]` }
];

export function KeyboardDesigner({ onSaveSuccess }: Props) {
    const [kleInput, setKleInput] = useState("");
    const [previewDef, setPreviewDef] = useState<KeyboardDefinition | null>(null);
    const [error, setError] = useState<string | null>(null);

    const [meta, setMeta] = useState({
        name: "My Custom Board",
        author: "Me",
        type: "column_staggered"
    });

    useEffect(() => {
        if (!kleInput.trim()) {
            setPreviewDef(null);
            setError(null);
            return;
        }

        const timer = setTimeout(async () => {
            try {
                const def = await invoke<KeyboardDefinition>("cmd_parse_kle", { json: kleInput });
                def.meta = { ...def.meta, ...meta };
                setPreviewDef(def);
                setError(null);
            } catch (e) {
                setError(`Parse Error: ${e}`);
                setPreviewDef(null);
            }
        }, 500);

        return () => clearTimeout(timer);
    }, [kleInput, meta.name, meta.author, meta.type]);

    const handleSave = async () => {
        if (!previewDef) return;

        try {
            const finalDef = { ...previewDef, meta: { ...previewDef.meta, ...meta } };

            await invoke("cmd_save_keyboard", {
                filename: meta.name.toLowerCase().replace(/\s+/g, '_'),
                def: finalDef
            });

            alert("Keyboard Saved! It is now available in the selector.");
            onSaveSuccess();
        } catch (e) {
            alert(`Save failed: ${e}`);
        }
    };

    const handleOpenEditor = async () => {
        try {
            await invoke('plugin:opener|open', { path: 'https://www.keyboard-layout-editor.com/' });
        } catch (e) {
            console.error("Failed to open link", e);
        }
    };

    return (
        <div className="flex h-full w-full">

            {/* CENTER: Preview Area */}
            <div className="flex-1 flex flex-col bg-[radial-gradient(ellipse_at_center,_var(--tw-gradient-stops))] from-slate-900/50 to-[#0B0F19] relative min-w-0">
                <div className="h-14 border-b border-slate-800/50 flex items-center px-6 bg-[#0B0F19]/95 backdrop-blur z-10 justify-between">
                    <span className="text-[10px] font-bold text-slate-500 uppercase tracking-widest">
                        Preview
                    </span>
                    {previewDef && (
                        <span className="text-[10px] px-2 py-1 bg-slate-800 rounded text-slate-300 border border-slate-700">
                            {previewDef.geometry.keys.length} Keys Detected
                        </span>
                    )}
                </div>

                <div className="flex-1 p-8 flex items-center justify-center overflow-hidden">
                    {error ? (
                        <div className="text-red-400 font-mono text-xs max-w-md text-center p-6 border border-red-900/50 bg-red-950/20 rounded-xl">
                            <span className="font-bold block mb-2">JSON Parsing Failed</span>
                            {error}
                        </div>
                    ) : previewDef ? (
                        <KeyboardMap
                            geometry={previewDef.geometry}
                            layoutString=""
                            className="w-full h-full max-w-4xl"
                        />
                    ) : (
                        <div className="text-slate-600 font-mono text-xs text-center p-8 border-2 border-dashed border-slate-800 rounded-xl">
                            Paste raw KLE JSON data to generate preview...
                        </div>
                    )}
                </div>
            </div>

            {/* RIGHT: Controls & Input */}
            <div className="w-96 bg-slate-900 border-l border-slate-800 flex flex-col shrink-0">
                <div className="p-4 border-b border-slate-800 flex items-center bg-slate-950/30">
                    <h3 className="text-xs font-bold text-slate-400 uppercase flex items-center gap-2">
                        <PenTool size={14} /> Construct
                    </h3>
                </div>

                <div className="flex-1 overflow-y-auto p-4 custom-scrollbar">
                    <div className="space-y-4 mb-8">
                        <div>
                            <Label>Name</Label>
                            <Input
                                value={meta.name}
                                onChange={e => setMeta({ ...meta, name: e.target.value })}
                            />
                        </div>
                        <div>
                            <Label>Author</Label>
                            <Input
                                value={meta.author}
                                onChange={e => setMeta({ ...meta, author: e.target.value })}
                            />
                        </div>
                        <div>
                            <Label>Type</Label>
                            <Select
                                value={meta.type}
                                onChange={e => setMeta({ ...meta, type: e.target.value })}
                                options={[
                                    { label: "Ortholinear", value: "ortho" },
                                    { label: "Column Stagger", value: "column_staggered" },
                                    { label: "Row Stagger", value: "row_staggered" }
                                ]}
                            />
                        </div>
                    </div>

                    <div className="flex flex-col h-80">
                        <div className="flex justify-between items-center mb-2">
                            <Label>Raw KLE JSON</Label>
                            <button
                                onClick={handleOpenEditor}
                                className="text-[10px] text-blue-400 hover:text-blue-300 hover:underline"
                            >
                                Open Editor ↗
                            </button>
                        </div>

                        <div className="mb-2">
                            <Select
                                options={TEMPLATES}
                                onChange={(e) => setKleInput(e.target.value)}
                                className="text-[10px]"
                            />
                        </div>

                        <textarea
                            className="flex-1 bg-slate-950/50 border border-slate-800 rounded-lg p-3 text-[10px] font-mono text-slate-400 outline-none focus:border-blue-500 resize-none transition-colors"
                            placeholder='["Q", "W", "E", ...]'
                            value={kleInput}
                            onChange={e => setKleInput(e.target.value)}
                        />
                    </div>

                    <div className="mt-6 border-t border-slate-800 pt-4">
                        <Button
                            variant="secondary"
                            className="w-full"
                            onClick={handleSave}
                            disabled={!previewDef}
                            icon={<Save size={16} />}
                        >
                            SAVE DEFINITION
                        </Button>
                    </div>
                </div>
            </div>
        </div>
    );
}