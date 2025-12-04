// ===== keyforge/ui/src/views/OptimizeView.tsx =====
import { useKeyboard } from "../context/KeyboardContext";
import { Inspector } from "../components/Inspector";
import { KeyboardMap } from "../components/KeyboardMap";
import { toDisplayString, fromDisplayString, formatForDisplay } from "../utils";
import { Button } from "../components/ui/Button";
import { Input } from "../components/ui/Input";
import { RefreshCw, ArrowRight } from "lucide-react";

interface Props {
    isSyncing: boolean;
    onSync: () => void;
    localWorkerEnabled: boolean;
    toggleWorker: (b: boolean) => void;
    pinnedKeys: string;
    setPinnedKeys: (s: string) => void;
    onDispatch: () => void; // Hive Dispatch
    onStopJob: () => void;  // Hive Stop
}

export function OptimizeView({
    isSyncing, onSync,
    localWorkerEnabled, toggleWorker, pinnedKeys, setPinnedKeys,
    onDispatch, onStopJob
}: Props) {
    const {
        activeResult, layoutName, layoutString, updateLayoutString,
        selectedKeyboard, activeJobId
    } = useKeyboard();

    const handleCommitInput = () => {
        const standardized = fromDisplayString(layoutString);
        updateLayoutString(formatForDisplay(standardized));
    };

    return (
        <>
            <div className="flex-1 flex flex-col min-w-0 relative bg-[radial-gradient(ellipse_at_center,_var(--tw-gradient-stops))] from-slate-900/50 to-[#0B0F19]">
                <div className="h-14 flex items-center px-6 border-b border-slate-800/50 justify-between bg-[#0B0F19]/90 backdrop-blur z-10">
                    <div className="flex items-center gap-2">
                        <h2 className="text-lg font-black text-white tracking-tight">{layoutName}</h2>
                        <span className="text-[10px] px-1.5 py-0.5 rounded bg-slate-800 text-slate-400 font-mono border border-slate-700/50">
                            {selectedKeyboard}
                        </span>
                        {activeJobId && <span className="text-xs text-purple-400 font-mono animate-pulse">OPTIMIZING...</span>}
                    </div>
                    <Button size="icon" variant="ghost" onClick={onSync} isLoading={isSyncing} icon={<RefreshCw size={18} />} />
                </div>

                <div className="flex-1 p-8 flex flex-col items-center justify-center">
                    <KeyboardMap
                        geometry={activeResult?.geometry}
                        layoutString={toDisplayString(fromDisplayString(layoutString))}
                        heatmap={activeResult?.heatmap}
                        className="w-full h-full max-w-4xl"
                    />

                    <div className="mt-8 w-full max-w-2xl flex gap-2">
                        <Input
                            className="text-center font-mono text-lg tracking-widest h-14"
                            value={layoutString}
                            onChange={e => updateLayoutString(e.target.value)}
                            onKeyDown={e => e.key === 'Enter' && handleCommitInput()}
                            onBlur={handleCommitInput}
                            disabled={!!activeJobId}
                            placeholder="Type keys..."
                        />
                        <Button variant="secondary" className="h-14 w-14" disabled={!!activeJobId} icon={<ArrowRight size={24} />} onClick={handleCommitInput} />
                    </div>
                </div>
            </div>

            <Inspector
                mode="optimize"
                onDispatch={onDispatch}
                onStop={onStopJob}
                localWorkerEnabled={localWorkerEnabled}
                toggleWorker={toggleWorker}
                pinnedKeys={pinnedKeys}
                setPinnedKeys={setPinnedKeys}
            />
        </>
    );
}