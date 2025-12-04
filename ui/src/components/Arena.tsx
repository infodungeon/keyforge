// ===== keyforge/ui/src/components/Arena.tsx =====
import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { RotateCcw, Keyboard as KeyboardIcon, Save, UserCheck } from "lucide-react";
import { Button } from "./ui/Button";
import { useToast } from "../context/ToastContext";
import { useLibrary } from "../context/LibraryContext"; // ADDED

// ... (KeyStroke/BiometricSample definitions same)
interface KeyStroke {
    char: string;
    timestamp: number;
}

interface BiometricSample {
    bigram: string;
    ms: number;
    timestamp: number;
}

export function Arena() {
    const { addToast } = useToast();
    const { refreshLibrary } = useLibrary(); // ADDED
    const [words, setWords] = useState<string[]>([]);
    const [input, setInput] = useState("");
    const [currentIndex, setCurrentIndex] = useState(0);

    // Stats
    const [startTime, setStartTime] = useState<number | null>(null);
    const [wpm, setWpm] = useState(0);
    const [acc, setAcc] = useState(100);
    const [isFinished, setIsFinished] = useState(false);
    const [isLoading, setIsLoading] = useState(false);
    const [saveStatus, setSaveStatus] = useState<string>("");
    const [isGenerating, setIsGenerating] = useState(false); // ADDED

    const inputRef = useRef<HTMLInputElement>(null);
    const lastStrokeRef = useRef<KeyStroke | null>(null);
    const biometricsRef = useRef<BiometricSample[]>([]);
    const errorsRef = useRef<number>(0);

    const generateWords = useCallback(async () => {
        setIsLoading(true);
        setSaveStatus("");
        try {
            const newWords = await invoke<string[]>("cmd_get_typing_words", { count: 50 });
            setWords(newWords);
            // Reset logic...
            setInput("");
            setCurrentIndex(0);
            setStartTime(null);
            setWpm(0);
            setAcc(100);
            setIsFinished(false);
            lastStrokeRef.current = null;
            biometricsRef.current = [];
            errorsRef.current = 0;
            setTimeout(() => inputRef.current?.focus(), 100);
        } catch (e) {
            console.error("Failed to load corpus:", e);
            addToast('error', "Could not load word list.");
        } finally {
            setIsLoading(false);
        }
    }, [addToast]);

    useEffect(() => {
        generateWords();
    }, [generateWords]);

    // ... (handleKeyDown, handleChange same)
    const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
        if (isFinished || isLoading) return;
        const now = performance.now();
        if (!startTime) setStartTime(now);
        const targetWord = words[currentIndex];
        const val = e.currentTarget.value;

        if (e.key.length === 1 && /[a-z]/i.test(e.key)) {
            if (lastStrokeRef.current) {
                const delta = now - lastStrokeRef.current.timestamp;
                if (delta < 2000) {
                    const bigram = (lastStrokeRef.current.char + e.key).toLowerCase();
                    biometricsRef.current.push({ bigram, ms: delta, timestamp: Date.now() });
                }
            }
            lastStrokeRef.current = { char: e.key, timestamp: now };
        }

        if (e.key === ' ') {
            e.preventDefault();
            if (val.trim() === targetWord) {
                setInput("");
                setCurrentIndex(prev => prev + 1);
                if (currentIndex >= words.length - 1) finishTest();
            }
        }
    };

    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        if (isFinished) return;
        const val = e.target.value;
        setInput(val);
        const targetWord = words[currentIndex];
        if (!targetWord.startsWith(val)) errorsRef.current += 1;
    };

    const finishTest = async () => {
        if (!startTime) return;
        const durationMin = (performance.now() - startTime) / 60000;
        const charCount = words.join(" ").length;
        const rawWpm = (charCount / 5) / durationMin;
        const accuracy = Math.max(0, 100 - ((errorsRef.current / charCount) * 100));

        setWpm(Math.round(rawWpm));
        setAcc(Math.round(accuracy));
        setIsFinished(true);

        if (biometricsRef.current.length > 0) {
            setSaveStatus("Saving Stats...");
            try {
                const msg = await invoke<string>("cmd_save_biometrics", { samples: biometricsRef.current });
                setSaveStatus("Stats Saved");
                addToast('success', msg, 3000);
            } catch (e) {
                console.error(e);
                setSaveStatus("Save Failed");
                addToast('error', `Failed to save stats: ${e}`);
            }
        }
    };

    // ADDED: Generate Profile Action
    const handleGenerateProfile = async () => {
        setIsGenerating(true);
        try {
            const msg = await invoke<string>("cmd_generate_personal_profile");
            addToast('success', msg);
            await refreshLibrary(); // Reload list so personal_cost.csv appears
        } catch (e) {
            addToast('error', `Profile Generation Failed: ${e}`);
        } finally {
            setIsGenerating(false);
        }
    };

    const renderWord = (word: string, idx: number) => {
        let className = "text-2xl font-mono tracking-wide px-1.5 rounded my-1 transition-colors ";
        if (idx < currentIndex) className += "text-slate-600";
        else if (idx === currentIndex) className += "text-slate-200 bg-slate-800/50";
        else className += "text-slate-700";

        if (idx === currentIndex) {
            return (
                <span key={idx} className={className}>
                    {word.split('').map((char, charIdx) => {
                        let charClass = "";
                        if (charIdx < input.length) {
                            charClass = input[charIdx] === char ? "text-slate-200" : "text-red-400";
                        }
                        const isCursor = charIdx === input.length;
                        return (
                            <span key={charIdx} className={`relative ${charClass}`}>
                                {isCursor && <span className="absolute -left-[1px] -top-1 bottom-0 w-[2px] bg-purple-500 animate-pulse" />}
                                {char}
                            </span>
                        );
                    })}
                    {input.length >= word.length && <span className="inline-block w-[2px] h-5 bg-purple-500 animate-pulse align-middle ml-[1px]" />}
                </span>
            );
        }
        return <span key={idx} className={className}>{word}</span>;
    };

    return (
        <div
            className="flex-1 flex flex-col items-center justify-center bg-[#0B0F19] relative overflow-hidden"
            onClick={() => inputRef.current?.focus()}
        >
            {/* Header / Stats */}
            <div className="absolute top-0 left-0 w-full p-6 flex justify-center gap-12 text-slate-500 font-mono text-sm uppercase tracking-widest select-none">
                {isFinished ? (
                    <>
                        <div className="flex flex-col items-center">
                            <span className="text-[10px] font-bold mb-1 text-slate-600">WPM</span>
                            <span className="text-4xl text-purple-400 font-black">{wpm}</span>
                        </div>
                        <div className="flex flex-col items-center">
                            <span className="text-[10px] font-bold mb-1 text-slate-600">ACC</span>
                            <span className="text-4xl text-blue-400 font-black">{acc}%</span>
                        </div>
                    </>
                ) : (
                    <div className="flex items-center gap-2 opacity-50">
                        <KeyboardIcon size={16} />
                        <span>Typing Test</span>
                    </div>
                )}
            </div>

            <input
                ref={inputRef}
                className="absolute opacity-0 pointer-events-none"
                value={input}
                onChange={handleChange}
                onKeyDown={handleKeyDown}
                autoFocus
                onBlur={() => { if (!isFinished) setTimeout(() => inputRef.current?.focus(), 10); }}
            />

            <div className="max-w-4xl w-full p-8 flex flex-wrap justify-center content-start gap-y-2 relative z-10 select-none cursor-text">
                {isLoading ? (
                    <div className="text-slate-500 animate-pulse">Loading Corpus...</div>
                ) : isFinished ? (
                    <div className="flex flex-col items-center gap-6 animate-in fade-in zoom-in duration-300">
                        <div className="text-xl text-slate-300 font-bold">Session Complete</div>

                        <div className={`text-xs font-mono flex items-center gap-2 ${saveStatus.includes("Failed") ? "text-red-400" : "text-green-400"}`}>
                            <Save size={14} /> {saveStatus}
                        </div>

                        <div className="flex gap-4 mt-4">
                            <Button onClick={generateWords} icon={<RotateCcw size={16} />}>Restart</Button>
                            {/* NEW BUTTON */}
                            <Button
                                variant="secondary"
                                onClick={handleGenerateProfile}
                                isLoading={isGenerating}
                                icon={<UserCheck size={16} />}
                            >
                                Generate Personal Profile
                            </Button>
                        </div>
                        <p className="text-xs text-slate-600">Biometrics captured: {biometricsRef.current.length} samples</p>
                    </div>
                ) : (
                    words.map((w, i) => renderWord(w, i))
                )}
            </div>

            {!isFinished && !isLoading && (
                <div className="absolute bottom-8 flex gap-4 opacity-50 hover:opacity-100 transition-opacity">
                    <button onClick={generateWords} className="p-2 bg-slate-800 hover:bg-slate-700 rounded-full text-slate-400 transition-colors" title="Restart Test">
                        <RotateCcw size={16} />
                    </button>
                    <div className="p-2 text-xs font-mono text-slate-500">
                        {currentIndex} / {words.length}
                    </div>
                </div>
            )}
        </div>
    );
}