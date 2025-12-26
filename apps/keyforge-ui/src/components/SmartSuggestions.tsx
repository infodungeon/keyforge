import { useState, useEffect } from "react";
import { Lightbulb, ArrowRight, Loader2 } from "lucide-react";
import { useSession } from "../context/SessionContext";
import { useToast } from "../context/ToastContext";
import { fromDisplayString, keycodeService } from "../utils";
import { useBackend } from "../context/BackendContext";

interface SwapSuggestion {
  index_a: number;
  index_b: number;
  key_a: string; // These come back as indices from backend
  key_b: string;
  score_delta: number;
  improvement_pct: number;
}

interface Props {
  onHover: (indices: number[] | null) => void;
}

export function SmartSuggestions({ onHover }: Props) {
  const { layoutString, updateLayoutString } = useSession();
  const { addToast } = useToast();
  const backend = useBackend();
  const [suggestions, setSuggestions] = useState<SwapSuggestion[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    if (!layoutString) return;

    const fetchSuggestions = async () => {
      setIsLoading(true);
      try {
        const qmkStr = fromDisplayString(layoutString);
        const data = await backend.getSmartSwaps(qmkStr);

        // Resolve labels locally since backend sends indices/raw
        const tokens = layoutString.trim().split(/\s+/);
        const resolved = data.map((s) => ({
          ...s,
          key_a: keycodeService.getVisualLabel(tokens[s.index_a] || ""),
          key_b: keycodeService.getVisualLabel(tokens[s.index_b] || ""),
        }));

        setSuggestions(resolved);
      } catch (e) {
        console.error("Failed to get suggestions:", e);
      } finally {
        setIsLoading(false);
      }
    };

    // Debounce
    const timer = setTimeout(fetchSuggestions, 500);
    return () => clearTimeout(timer);
  }, [layoutString, backend]);

  const applySwap = (s: SwapSuggestion) => {
    const tokens = layoutString.trim().split(/\s+/);
    if (s.index_a < tokens.length && s.index_b < tokens.length) {
      const temp = tokens[s.index_a];
      tokens[s.index_a] = tokens[s.index_b];
      tokens[s.index_b] = temp;
      updateLayoutString(tokens.join(" "));
      addToast("success", `Swapped ${s.key_a} and ${s.key_b}`);
      onHover(null); // Clear highlight
    }
  };

  if (suggestions.length === 0 && !isLoading) return null;

  return (
    <div className="bg-gradient-to-br from-indigo-900/40 to-purple-900/40 border border-indigo-500/30 rounded-xl p-4 mb-6">
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2">
          <div className="p-1.5 bg-indigo-500/20 rounded-lg text-indigo-300">
            <Lightbulb size={14} />
          </div>
          <h4 className="text-xs font-bold text-indigo-200 uppercase tracking-wide">
            Smart Assist
          </h4>
        </div>
        {isLoading && (
          <Loader2 size={12} className="animate-spin text-indigo-400" />
        )}
      </div>

      <div className="space-y-2">
        {suggestions.slice(0, 3).map((s, i) => (
          <div
            key={i}
            onClick={() => applySwap(s)}
            onMouseEnter={() => onHover([s.index_a, s.index_b])}
            onMouseLeave={() => onHover(null)}
            className="flex items-center justify-between bg-slate-900/50 p-2 rounded border border-indigo-500/20 hover:border-indigo-500/50 transition-colors group cursor-pointer"
          >
            <div className="flex items-center gap-3">
              <div className="flex gap-1 font-mono text-xs font-bold text-white">
                <span className="bg-slate-800 px-1.5 rounded min-w-[1.5rem] text-center">
                  {s.key_a}
                </span>
                <ArrowRight size={12} className="text-slate-500 mt-1" />
                <span className="bg-slate-800 px-1.5 rounded min-w-[1.5rem] text-center">
                  {s.key_b}
                </span>
              </div>
              <div className="text-[10px] text-slate-400">
                Improves by{" "}
                <span className="text-green-400 font-bold">
                  {s.improvement_pct.toFixed(2)}%
                </span>
              </div>
            </div>
            <button className="text-[10px] bg-indigo-600 hover:bg-indigo-500 text-white px-2 py-1 rounded opacity-0 group-hover:opacity-100 transition-opacity">
              Apply
            </button>
          </div>
        ))}
        {suggestions.length === 0 && !isLoading && (
          <div className="text-[10px] text-slate-500 italic text-center">
            No obvious improvements found. Good job!
          </div>
        )}
      </div>
    </div>
  );
}
