// ===== keyforge/ui/src/context/ArenaContext.tsx =====
import { createContext, useContext, useState, useEffect, useRef, useCallback, ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "./ToastContext";
import { useLibrary } from "./LibraryContext";
import { coverageService } from "../services/coverage";
import { BiometricSample } from "../types"; // ADDED

export const ZOOM_LEVELS = [
    "text-sm", "text-[15px]", "text-base", "text-[17px]",
    "text-lg", "text-[19px]", "text-xl", "text-[22px]",
    "text-2xl", "text-[27px]", "text-3xl", "text-[33px]",
    "text-4xl", "text-[42px]", "text-5xl"
];
const DEFAULT_ZOOM = 8;

interface ArenaContextType {
    // State
    words: string[];
    input: string;
    currentIndex: number;
    wpm: number;
    accuracy: number;
    isFinished: boolean;
    isLoading: boolean;
    isGenerating: boolean;
    coveragePct: number;
    sampleCount: number;
    stopOnError: boolean;
    zoomIndex: number;

    // Actions
    setInput: (s: string) => void;
    setStopOnError: (b: boolean) => void;
    changeZoom: (delta: number) => void;
    nextSession: () => void;
    resetData: () => Promise<void>;
    generateProfile: () => Promise<void>;
    handleKeyDown: (e: React.KeyboardEvent<HTMLInputElement>) => void;
    handleChange: (e: React.ChangeEvent<HTMLInputElement>) => void;

    // Refs
    inputRef: React.RefObject<HTMLInputElement | null>;
}

const ArenaContext = createContext<ArenaContextType | undefined>(undefined);

export function ArenaProvider({ children }: { children: ReactNode }) {
    const { addToast } = useToast();
    const { refreshLibrary, selectedCorpus } = useLibrary();

    // --- State ---
    const [wordPool, setWordPool] = useState<string[]>([]);
    const [words, setWords] = useState<string[]>([]);
    const [input, setInput] = useState("");
    const [currentIndex, setCurrentIndex] = useState(0);
    const [stopOnError, setStopOnError] = useState(false);
    const [zoomIndex, setZoomIndex] = useState(() => {
        const saved = localStorage.getItem("keyforge_arena_zoom");
        return saved ? Math.max(0, Math.min(parseInt(saved, 10), ZOOM_LEVELS.length - 1)) : DEFAULT_ZOOM;
    });

    // Performance State
    const [startTime, setStartTime] = useState<number | null>(null);
    const [wpm, setWpm] = useState(0);
    const [acc, setAcc] = useState(100);
    const [isFinished, setIsFinished] = useState(false);
    const [isLoading, setIsLoading] = useState(false);
    const [isGenerating, setIsGenerating] = useState(false);
    const [coverage, setCoverage] = useState(0);
    const [sampleCount, setSampleCount] = useState(0);

    const inputRef = useRef<HTMLInputElement>(null);

    // Private refs
    const lastStrokeRef = useRef<{ char: string; timestamp: number } | null>(null);
    const biometricsRef = useRef<any[]>([]);
    const errorsRef = useRef<number>(0);
    const currentWordErrorRef = useRef<boolean>(false);

    // --- Effects ---
    useEffect(() => {
        localStorage.setItem("keyforge_arena_zoom", zoomIndex.toString());
    }, [zoomIndex]);

    // Load Data
    useEffect(() => {
        const load = async () => {
            try {
                // 1. Load Words
                const pool = await invoke<string[]>("cmd_get_typing_words", { count: 2000 });
                setWordPool(pool);

                // 2. Load Dynamic Targets from Corpus
                const corpusFile = selectedCorpus || "ngrams-all.tsv";
                const targets = await invoke<string[]>("cmd_get_corpus_bigrams", {
                    corpusFilename: corpusFile,
                    limit: 100
                });
                coverageService.setTargets(targets);

                // 3. Hydrate Existing Stats (NEW)
                try {
                    const history = await invoke<BiometricSample[]>("cmd_load_user_stats");
                    if (history && history.length > 0) {
                        coverageService.hydrateHistory(history);
                    }
                } catch (e) {
                    console.warn("Could not load history:", e);
                }

                // 4. Initialize Session
                const initialSet = coverageService.selectTargetedWords(pool, 50);
                setWords(initialSet);
                setCoverage(coverageService.getStats().coveragePct);
                setSampleCount(coverageService.getStats().totalSamples);

            } catch (e) {
                console.error(e);
                addToast('error', "Failed to load Arena data");
            }
        };
        load();
    }, [addToast, selectedCorpus]);

    // --- Logic ---
    const changeZoom = (delta: number) => {
        setZoomIndex(prev => Math.min(Math.max(0, prev + delta), ZOOM_LEVELS.length - 1));
    };

    const nextSession = useCallback(() => {
        if (wordPool.length === 0) return;
        setIsLoading(true);
        const newWords = coverageService.selectTargetedWords(wordPool, 50);
        setWords(newWords);
        setInput("");
        setCurrentIndex(0);
        setStartTime(null);
        setWpm(0);
        setAcc(100);
        setIsFinished(false);
        lastStrokeRef.current = null;
        biometricsRef.current = [];
        errorsRef.current = 0;
        currentWordErrorRef.current = false;
        setTimeout(() => inputRef.current?.focus(), 100);
        setIsLoading(false);
    }, [wordPool]);

    const finishTest = async () => {
        if (!startTime) return;
        const durationMin = (performance.now() - startTime) / 60000;
        const totalChars = words.join(" ").length;
        const rawWpm = (totalChars / 5) / durationMin;
        const accuracy = Math.max(0, 100 - ((errorsRef.current / totalChars) * 100));

        setWpm(Math.round(rawWpm));
        setAcc(Math.round(accuracy));
        setIsFinished(true);

        if (biometricsRef.current.length > 0) {
            try {
                await invoke("cmd_save_biometrics", { samples: biometricsRef.current });
                // We add current session to service so UI reflects it immediately without reload
                // BUT we need to convert to simple struct
                const simpleSamples = biometricsRef.current.map(b => ({ bigram: b.bigram }));
                coverageService.hydrateHistory(simpleSamples);

                const stats = coverageService.getStats();
                setCoverage(stats.coveragePct);
                setSampleCount(stats.totalSamples);

                addToast('success', `Saved ${biometricsRef.current.length} samples`, 2000);
            } catch (e) {
                addToast('error', `Failed to save stats: ${e}`);
            }
        }
    };

    const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
        if (e.ctrlKey) {
            if (['ArrowUp', '=', '+'].includes(e.key)) { e.preventDefault(); changeZoom(1); return; }
            if (['ArrowDown', '-'].includes(e.key)) { e.preventDefault(); changeZoom(-1); return; }
        }

        if (isFinished || isLoading) return;

        const now = performance.now();
        if (!startTime) setStartTime(now);

        const targetWord = words[currentIndex];
        const val = e.currentTarget.value;

        if (e.key === ' ') {
            e.preventDefault();
            const isCorrect = val.trim() === targetWord;
            if (stopOnError && !isCorrect) return;

            if (isCorrect) {
                // We register input for word selection weighting
                coverageService.registerInput(targetWord);
            } else {
                errorsRef.current += 1;
            }

            setInput("");
            currentWordErrorRef.current = false;
            setCurrentIndex(prev => prev + 1);
            if (currentIndex >= words.length - 1) finishTest();
            return;
        }

        if (e.key.length === 1 && /[a-z.,';]/i.test(e.key) && !e.ctrlKey && !e.metaKey) {
            const isPrefixMatch = targetWord.startsWith(val + e.key);
            if (isPrefixMatch && !currentWordErrorRef.current) {
                if (lastStrokeRef.current) {
                    const delta = now - lastStrokeRef.current.timestamp;
                    if (delta < 2000) {
                        const bigram = (lastStrokeRef.current.char + e.key).toLowerCase();
                        biometricsRef.current.push({ bigram, ms: delta, timestamp: Date.now() });
                    }
                }
                lastStrokeRef.current = { char: e.key, timestamp: now };
            } else {
                currentWordErrorRef.current = true;
            }
        }
    };

    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        if (isFinished) return;
        const val = e.target.value;
        setInput(val);
        const targetWord = words[currentIndex];
        if (!targetWord.startsWith(val)) {
            currentWordErrorRef.current = true;
        }
    };

    const resetData = async () => {
        try {
            await invoke("cmd_reset_user_stats");
            coverageService.reset();
            setCoverage(0);
            setSampleCount(0);
            biometricsRef.current = [];
            addToast('success', "Biometric data cleared.");
            nextSession();
        } catch (e) {
            addToast('error', `Reset failed: ${e}`);
        }
    };

    const generateProfile = async () => {
        setIsGenerating(true);
        try {
            const msg = await invoke<string>("cmd_generate_personal_profile");
            addToast('success', msg);
            await refreshLibrary();
        } catch (e) {
            addToast('error', `Generation Failed: ${e}`);
        } finally {
            setIsGenerating(false);
        }
    };

    return (
        <ArenaContext.Provider value={{
            words, input, currentIndex, wpm, accuracy: acc, isFinished,
            isLoading, isGenerating, coveragePct: coverage, sampleCount,
            stopOnError, zoomIndex,
            inputRef,
            setInput, setStopOnError, changeZoom, nextSession,
            resetData, generateProfile, handleKeyDown, handleChange
        }}>
            {children}
        </ArenaContext.Provider>
    );
}

export const useArena = () => {
    const ctx = useContext(ArenaContext);
    if (!ctx) throw new Error("useArena must be used within ArenaProvider");
    return ctx;
};