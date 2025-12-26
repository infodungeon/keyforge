import {
  createContext,
  useContext,
  useState,
  useEffect,
  useRef,
  useCallback,
  ReactNode,
} from "react";
import { useToast } from "./ToastContext";
import { useLibrary } from "./LibraryContext";
import { useBackend } from "./BackendContext";
import { coverageService } from "../services/coverage";
import { CorpusSource } from "../types"; // Import CorpusSource

export const ZOOM_LEVELS = [
  "text-sm",
  "text-[15px]",
  "text-base",
  "text-[17px]",
  "text-lg",
  "text-[19px]",
  "text-xl",
  "text-[22px]",
  "text-2xl",
  "text-[27px]",
  "text-3xl",
  "text-[33px]",
  "text-4xl",
  "text-[42px]",
  "text-5xl",
];
const DEFAULT_ZOOM = 8;

interface ArenaContextType {
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
  inputRef: React.RefObject<HTMLInputElement | null>;
  setInput: (s: string) => void;
  setStopOnError: (b: boolean) => void;
  changeZoom: (delta: number) => void;
  nextSession: () => void;
  resetData: () => Promise<void>;
  generateProfile: () => Promise<void>;
  handleKeyDown: (e: React.KeyboardEvent<HTMLInputElement>) => void;
  handleChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
}

const ArenaContext = createContext<ArenaContextType | undefined>(undefined);

// Helper to parse "name:weight,name:weight"
const parseCorporaStr = (str: string): CorpusSource[] => {
  return str.split(",").map((s) => {
    const [id, w] = s.trim().split(":");
    return {
      id: id.trim(),
      weight: w ? parseFloat(w) : 1.0,
    };
  });
};

export function ArenaProvider({ children }: { children: ReactNode }) {
  const { addToast } = useToast();
  const { selectedCorpus } = useLibrary();
  const backend = useBackend();

  const [wordPool, setWordPool] = useState<string[]>([]);
  const [words, setWords] = useState<string[]>([]);
  const [input, setInput] = useState("");
  const [currentIndex, setCurrentIndex] = useState(0);
  const [stopOnError, setStopOnError] = useState(false);
  const [zoomIndex, setZoomIndex] = useState(() => {
    const saved = localStorage.getItem("keyforge_arena_zoom");
    return saved
      ? Math.max(0, Math.min(parseInt(saved, 10), ZOOM_LEVELS.length - 1))
      : DEFAULT_ZOOM;
  });

  const [startTime, setStartTime] = useState<number | null>(null);
  const [wpm, setWpm] = useState(0);
  const [acc, setAcc] = useState(100);
  const [isFinished, setIsFinished] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [isGenerating, setIsGenerating] = useState(false);
  const [coverage, setCoverage] = useState(0);
  const [sampleCount, setSampleCount] = useState(0);

  const inputRef = useRef<HTMLInputElement>(null);
  const lastStrokeRef = useRef<{ char: string; timestamp: number } | null>(
    null,
  );
  const biometricsRef = useRef<any[]>([]);
  const errorsRef = useRef<number>(0);
  const currentWordErrorRef = useRef<boolean>(false);

  useEffect(() => {
    localStorage.setItem("keyforge_arena_zoom", zoomIndex.toString());
  }, [zoomIndex]);

  // Load Data whenever the selected corpus changes
  useEffect(() => {
    if (!selectedCorpus) return;

    const load = async () => {
      try {
        setIsLoading(true);
        const corpora = parseCorporaStr(selectedCorpus || "text/en_std");

        // 1. Load Words from the mixed corpus
        const pool = await backend.getTypingWords(corpora, 2000);
        setWordPool(pool);

        // 2. Load Dynamic Targets
        const targets = await backend.getCorpusBigrams(corpora, 100);
        coverageService.setTargets(targets);

        // 3. Hydrate Existing Stats
        try {
          const history = await backend.loadUserStats();
          if (history && history.length > 0) {
            coverageService.hydrateHistory(history);
          }
        } catch (e) {
          console.warn("Could not load history:", e);
        }

        const stats = coverageService.getStats();
        setCoverage(stats.coveragePct);
        setSampleCount(stats.totalSamples);

        // Initialize first session immediately
        if (pool.length > 0) {
          const initialSet = coverageService.selectTargetedWords(pool, 50);
          setWords(initialSet);
        }
      } catch (e) {
        console.error(e);
        addToast("error", `Failed to load Arena data: ${e}`);
      } finally {
        setIsLoading(false);
      }
    };
    load();
  }, [addToast, selectedCorpus]); // Re-run when user changes corpus in settings

  const changeZoom = (delta: number) => {
    setZoomIndex((prev) =>
      Math.min(Math.max(0, prev + delta), ZOOM_LEVELS.length - 1),
    );
  };

  const nextSession = useCallback(() => {
    if (wordPool.length === 0) return;
    setIsLoading(true);
    // Small delay to allow UI to render loading state
    setTimeout(() => {
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
      setIsLoading(false);
      setTimeout(() => inputRef.current?.focus(), 50);
    }, 10);
  }, [wordPool]);

  const finishTest = async () => {
    if (!startTime) return;
    const durationMin = (performance.now() - startTime) / 60000;
    const totalChars = words.join(" ").length;
    const rawWpm = totalChars / 5 / durationMin;
    const accuracy = Math.max(0, 100 - (errorsRef.current / totalChars) * 100);

    setWpm(Math.round(rawWpm));
    setAcc(Math.round(accuracy));
    setIsFinished(true);

    if (biometricsRef.current.length > 0) {
      try {
        await backend.saveBiometrics(biometricsRef.current);
        const simpleSamples = biometricsRef.current.map((b) => ({
          bigram: b.bigram,
        }));
        coverageService.hydrateHistory(simpleSamples);

        const stats = coverageService.getStats();
        setCoverage(stats.coveragePct);
        setSampleCount(stats.totalSamples);

        addToast(
          "success",
          `Saved ${biometricsRef.current.length} samples`,
          2000,
        );
      } catch (e) {
        addToast("error", `Failed to save stats: ${e}`);
      }
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.ctrlKey) {
      if (["ArrowUp", "=", "+"].includes(e.key)) {
        e.preventDefault();
        changeZoom(1);
        return;
      }
      if (["ArrowDown", "-"].includes(e.key)) {
        e.preventDefault();
        changeZoom(-1);
        return;
      }
    }

    if (isFinished || isLoading) return;

    const now = performance.now();
    if (!startTime) setStartTime(now);

    const targetWord = words[currentIndex];
    const val = e.currentTarget.value;

    if (e.key === " ") {
      e.preventDefault();
      const isCorrect = val.trim() === targetWord;
      if (stopOnError && !isCorrect) return;

      if (isCorrect) {
        coverageService.registerInput(targetWord);
      } else {
        errorsRef.current += 1;
      }

      setInput("");
      currentWordErrorRef.current = false;
      setCurrentIndex((prev) => prev + 1);
      if (currentIndex >= words.length - 1) finishTest();
      return;
    }

    if (
      e.key.length === 1 &&
      /[a-z.,';]/i.test(e.key) &&
      !e.ctrlKey &&
      !e.metaKey
    ) {
      const isPrefixMatch = targetWord.startsWith(val + e.key);
      if (isPrefixMatch && !currentWordErrorRef.current) {
        if (lastStrokeRef.current) {
          const delta = now - lastStrokeRef.current.timestamp;
          if (delta < 2000) {
            const bigram = (lastStrokeRef.current.char + e.key).toLowerCase();
            biometricsRef.current.push({
              bigram,
              ms: delta,
              timestamp: Date.now(),
            });
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
      await backend.resetUserStats();
      coverageService.reset();
      setCoverage(0);
      setSampleCount(0);
      biometricsRef.current = [];
      addToast("success", "Biometric data cleared.");
      nextSession();
    } catch (e) {
      addToast("error", `Reset failed: ${e}`);
    }
  };

  const generateProfile = async () => {
    setIsGenerating(true);
    try {
      const msg = await backend.generatePersonalProfile();
      addToast("success", msg);
    } catch (e) {
      addToast("error", `Generation Failed: ${e}`);
    } finally {
      setIsGenerating(false);
    }
  };

  return (
    <ArenaContext.Provider
      value={{
        words,
        input,
        currentIndex,
        wpm,
        accuracy: acc,
        isFinished,
        isLoading,
        isGenerating,
        coveragePct: coverage,
        sampleCount,
        stopOnError,
        zoomIndex,
        inputRef,
        setInput,
        setStopOnError,
        changeZoom,
        nextSession,
        resetData,
        generateProfile,
        handleKeyDown,
        handleChange,
      }}
    >
      {children}
    </ArenaContext.Provider>
  );
}

export const useArena = () => {
  const ctx = useContext(ArenaContext);
  if (!ctx) throw new Error("useArena must be used within ArenaProvider");
  return ctx;
};
