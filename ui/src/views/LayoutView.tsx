import { useKeyboard } from "../context/KeyboardContext";
import { KeyboardMap } from "../components/KeyboardMap";
import { KeyPicker } from "../components/KeyPicker";
import { Button } from "../components/ui/Button";
import { Input } from "../components/ui/Input";
import { toDisplayString, fromDisplayString, formatForDisplay } from "../utils";
import { RefreshCw, ArrowRight } from "lucide-react";
import { useState, useEffect } from "react";

interface Props {
    isSyncing: boolean;
    onSync: () => void;
}

export function LayoutView({ isSyncing, onSync }: Props) {
    const {
        activeResult, layoutName, layoutString,
        updateLayoutString, selectedKeyboard,
        keyboards, selectKeyboard, availableLayouts, loadLayoutPreset,
        selectedKeyIndex, setSelectedKeyIndex
    } = useKeyboard();

    const [isEditingKey, setIsEditingKey] = useState(false);

    const handleCommitInput = () => {
        const standardized = fromDisplayString(layoutString);
        updateLayoutString(formatForDisplay(standardized));
    };

    const handleInsertToken = (token: string) => {
        if (selectedKeyIndex !== null && isEditingKey) {
            const tokens = layoutString.trim().split(/\s+/);
            const maxKeys = activeResult?.geometry.keys.length || 0;
            while (tokens.length < maxKeys) tokens.push("KC_TRNS");

            if (selectedKeyIndex < tokens.length) {
                tokens[selectedKeyIndex] = token;
                updateLayoutString(tokens.join(" "));

                if (selectedKeyIndex < maxKeys - 1) {
                    setSelectedKeyIndex(selectedKeyIndex + 1);
                } else {
                    setIsEditingKey(false);
                }
            }
        } else if (selectedKeyIndex === null) {
            updateLayoutString(layoutString + " " + token);
        }
    };

    useEffect(() => {
        if (selectedKeyIndex === null) setIsEditingKey(false);
    }, [selectedKeyIndex]);

    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (selectedKeyIndex !== null) {
                if (e.key === 'Tab') {
                    e.preventDefault(); e.stopPropagation();
                    setIsEditingKey(false);
                    const max = (activeResult?.geometry.keys.length || 1) - 1;
                    const dir = e.shiftKey ? -1 : 1;
                    let next = selectedKeyIndex + dir;
                    if (next < 0) next = max;
                    if (next > max) next = 0;
                    setSelectedKeyIndex(next);
                    return;
                }
                if (e.key === 'Escape') {
                    e.preventDefault();
                    if (isEditingKey) setIsEditingKey(false); else setSelectedKeyIndex(null);
                    return;
                }
                if (e.key === 'Enter') {
                    e.preventDefault();
                    setIsEditingKey(!isEditingKey);
                    return;
                }
            }
            if (isEditingKey && selectedKeyIndex !== null) {
                if (e.key.length === 1 && !e.ctrlKey && !e.metaKey && !e.altKey) {
                    handleInsertToken(e.key);
                }
            }
        };
        window.addEventListener('keydown', handleKeyDown as any);
        return () => window.removeEventListener('keydown', handleKeyDown as any);
    }, [selectedKeyIndex, isEditingKey, layoutString, activeResult]);

    const handleBackgroundClick = (e: React.MouseEvent) => {
        if (e.target === e.currentTarget) {
            setSelectedKeyIndex(null);
            setIsEditingKey(false);
        }
    };

    return (
        <>
            {/* CENTER: Visualization */}
            <div
                className="flex-1 flex flex-col min-w-0 relative bg-[radial-gradient(ellipse_at_center,_var(--tw-gradient-stops))] from-slate-900/50 to-[#0B0F19]"
                onClick={handleBackgroundClick}
            >
                <div className="h-14 flex items-center px-6 border-b border-slate-800/50 justify-between bg-[#0B0F19]/90 backdrop-blur z-10">
                    <div className="flex items-center gap-2">
                        <h2 className="text-lg font-black text-white tracking-tight">{layoutName}</h2>
                        <span className="text-[10px] px-1.5 py-0.5 rounded bg-slate-800 text-slate-400 font-mono border border-slate-700/50">
                            {selectedKeyboard}
                        </span>
                        {selectedKeyIndex !== null && (
                            <span className={`text-[10px] font-bold px-2 py-0.5 rounded border transition-colors ${isEditingKey ? "text-white bg-blue-600 border-blue-500" : "text-blue-400 bg-blue-900/30 border-blue-800"}`}>
                                {isEditingKey ? "EDITING" : `KEY #${selectedKeyIndex + 1}`}
                            </span>
                        )}
                    </div>
                    <div className="flex gap-2">
                        {/* REMOVED TEST KEYS BUTTON */}
                        <Button size="icon" variant="ghost" onClick={onSync} isLoading={isSyncing} icon={<RefreshCw size={18} />} />
                    </div>
                </div>

                <div className="flex-1 p-8 flex flex-col items-center justify-center" onClick={handleBackgroundClick}>
                    <KeyboardMap
                        geometry={activeResult?.geometry}
                        layoutString={toDisplayString(fromDisplayString(layoutString))}
                        heatmap={activeResult?.heatmap}
                        className="w-full h-full max-w-4xl"
                        selectedKeyIndex={selectedKeyIndex}
                        isEditing={isEditingKey}
                        onKeyClick={(i) => {
                            if (selectedKeyIndex === i) setIsEditingKey(true);
                            else { setSelectedKeyIndex(i); setIsEditingKey(false); }
                        }}
                    />

                    <div className="mt-8 w-full max-w-2xl flex gap-2" onClick={(e) => e.stopPropagation()}>
                        <Input
                            className="text-center font-mono text-lg tracking-widest h-14"
                            value={layoutString}
                            onChange={e => updateLayoutString(e.target.value)}
                            onKeyDown={e => e.key === 'Enter' && handleCommitInput()}
                            onBlur={handleCommitInput}
                            onFocus={() => { setSelectedKeyIndex(null); setIsEditingKey(false); }}
                            placeholder="Select a key or type code..."
                        />
                        <Button variant="secondary" className="h-14 w-14" icon={<ArrowRight size={24} />} onClick={handleCommitInput} />
                    </div>
                    <div className="text-[9px] text-slate-500 text-center mt-3 font-mono">
                        {isEditingKey
                            ? "Type to replace. TAB to next."
                            : selectedKeyIndex !== null
                                ? "ENTER to edit. TAB to move. ESC to deselect."
                                : "Click a key to select."}
                    </div>
                </div>
            </div>

            {/* RIGHT: Key Picker */}
            <KeyPicker
                onInsert={handleInsertToken}
                keyboards={keyboards}
                selectedKeyboard={selectedKeyboard}
                onSelectKeyboard={selectKeyboard}
                availableLayouts={availableLayouts}
                layoutName={layoutName}
                onSelectLayout={loadLayoutPreset}
            />
        </>
    );
}