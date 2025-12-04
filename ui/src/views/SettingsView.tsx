// ===== keyforge/ui/src/views/SettingsView.tsx =====
import { useKeyboard } from "../context/KeyboardContext";
import { useToast } from "../context/ToastContext";
import { Card } from "../components/ui/Card";
import { Label } from "../components/ui/Label";
import { Input } from "../components/ui/Input";
import { Select } from "../components/ui/Select";
import { Button } from "../components/ui/Button";
import { FileText } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

interface Props {
    hiveUrl: string;
    setHiveUrl: (u: string) => void;
    localWorkerEnabled: boolean;
    toggleWorker: (b: boolean) => void;
}

export function SettingsView({ hiveUrl, setHiveUrl, localWorkerEnabled, toggleWorker }: Props) {
    const {
        keyboards, corpora, selectedCorpus, selectCorpus, refreshData,
        // Now available via useKeyboard:
        costMatrices, selectedCostMatrix, selectCostMatrix
    } = useKeyboard();

    const { addToast } = useToast();

    const handleImportCorpus = async () => {
        try {
            const selected = await open({
                multiple: false,
                filters: [{ name: 'Text File', extensions: ['txt', 'md', 'rs', 'js', 'py', 'c'] }]
            });

            if (!selected) return;

            const name = prompt("Name this corpus (e.g. 'rust-code', 'novel'):");
            if (!name) return;

            const safeName = name.replace(/[^a-zA-Z0-9_-]/g, "_").toLowerCase();

            await invoke("cmd_import_corpus", {
                filePath: selected,
                name: safeName
            });

            addToast('success', "Corpus imported successfully.");
            await refreshData();

        } catch (e) {
            addToast('error', `Import failed: ${e}`);
        }
    };

    return (
        <div className="flex-1 p-12 bg-[#0B0F19] overflow-y-auto">
            <h2 className="text-2xl font-bold text-white mb-8">Settings</h2>

            <div className="grid grid-cols-2 gap-8 max-w-4xl">

                {/* DATA SETTINGS */}
                <Card>
                    <h3 className="text-lg font-bold text-white mb-4">Analysis Context</h3>
                    <div className="space-y-6">
                        {/* Corpus Selection */}
                        <div>
                            <Label>N-Gram Corpus</Label>
                            <div className="flex gap-2">
                                <div className="flex-1">
                                    <Select
                                        value={selectedCorpus}
                                        onChange={e => selectCorpus(e.target.value)}
                                        options={corpora.map((c: string) => ({ label: c, value: c }))}
                                    />
                                </div>
                                <Button variant="secondary" onClick={handleImportCorpus} icon={<FileText size={14} />}>
                                    Import
                                </Button>
                            </div>
                            <p className="text-[10px] text-slate-500 mt-1">
                                Text source used to calculate character and bigram frequencies.
                            </p>
                        </div>

                        {/* Cost Matrix Selection */}
                        <div>
                            <Label>Cost Matrix (Physics)</Label>
                            <Select
                                value={selectedCostMatrix}
                                onChange={e => selectCostMatrix(e.target.value)}
                                // FIXED: Explicit type annotation for 'c'
                                options={costMatrices.map((c: string) => ({ label: c, value: c }))}
                            />
                            <p className="text-[10px] text-slate-500 mt-1">
                                Determines how difficult specific key combinations are. Use 'personal_cost.csv' if you have run the Typing Arena.
                            </p>
                        </div>
                    </div>
                </Card>

                {/* NETWORK SETTINGS */}
                <Card>
                    <h3 className="text-lg font-bold text-white mb-4">Network</h3>
                    <div className="space-y-4">
                        <div>
                            <Label>Hive Server URL</Label>
                            <Input value={hiveUrl} onChange={e => setHiveUrl(e.target.value)} />
                            <p className="text-[10px] text-slate-500 mt-1">
                                The central server for distributed optimization and layout storage.
                            </p>
                        </div>
                    </div>
                </Card>

                {/* WORKER SETTINGS */}
                <Card>
                    <h3 className="text-lg font-bold text-white mb-4">Local Worker</h3>
                    <div className="flex items-center justify-between">
                        <div>
                            <Label>Enable Background Processing</Label>
                            <p className="text-[10px] text-slate-500">
                                Allows your machine to process jobs for the Hive when idle.
                            </p>
                        </div>
                        <input
                            type="checkbox"
                            checked={localWorkerEnabled}
                            onChange={e => toggleWorker(e.target.checked)}
                            className="accent-purple-500 h-5 w-5"
                        />
                    </div>
                </Card>

                {/* INFO CARD */}
                <Card>
                    <h3 className="text-lg font-bold text-white mb-4">System Info</h3>
                    <div className="space-y-2 text-xs text-slate-400">
                        <div className="flex justify-between">
                            <span>Keyboards Loaded</span>
                            <span className="font-mono text-white">{keyboards.length}</span>
                        </div>
                        <div className="flex justify-between">
                            <span>Corpora Loaded</span>
                            <span className="font-mono text-white">{corpora.length}</span>
                        </div>
                        <div className="flex justify-between">
                            <span>Matrices Loaded</span>
                            <span className="font-mono text-white">{costMatrices.length}</span>
                        </div>
                        <div className="flex justify-between">
                            <span>Client Version</span>
                            <span className="font-mono text-white">0.7.1</span>
                        </div>
                    </div>
                </Card>
            </div>
        </div>
    );
}