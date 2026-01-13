import {
  createContext,
  useContext,
  useState,
  useEffect,
  ReactNode,
  useRef,
} from "react";
import { useToast } from "./ToastContext";
import { ValidationResult } from "../types";
import { useSession } from "./SessionContext";
import { useLibrary } from "./LibraryContext";
import { useBackend } from "./BackendContext";
import { useSystem } from "./SystemContext";
import { fromDisplayString } from "../utils";

interface AnalysisContextType {
  activeResult: ValidationResult | null;
  referenceResult: ValidationResult | null;
  heatmap: number[] | undefined;
  isValidating: boolean;
}

const AnalysisContext = createContext<AnalysisContextType | undefined>(
  undefined,
);

export function AnalysisProvider({ children }: { children: ReactNode }) {
  const { layoutString, layoutName, activeJobId, isDatasetLoaded } = useSession();
  const { weights, selectedKeyboard } = useLibrary();
  const { hiveUrl } = useSystem();
  const backend = useBackend();
  const { addToast } = useToast();

  const [activeResult, setActiveResult] = useState<ValidationResult | null>(
    null,
  );
  // Reference result is now null by default as per user request to remove auto-calculation
  const [referenceResult] = useState<ValidationResult | null>(null);
  const [isValidating, setIsValidating] = useState(false);

  const validationReqId = useRef(0);
  const debounceTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Run Validation when layout changes
  useEffect(() => {
    if (!isDatasetLoaded) return;
    if (!layoutString) {
      setActiveResult(null);
      return;
    }
    if (activeJobId) return; 

    const run = async () => {
      const currentId = ++validationReqId.current;
      setIsValidating(true);
      try {
        const qmkStr = fromDisplayString(layoutString);
        const res = await backend.validateLayout(
          qmkStr,
          weights || undefined,
          hiveUrl,
          selectedKeyboard,
        );
        if (currentId === validationReqId.current) {
          setActiveResult({ ...res, layout_name: layoutName });
        }
      } catch (e) {
        addToast("error", `Analysis Failed: ${e instanceof Error ? e.message : String(e)}`);
      } finally {
        if (currentId === validationReqId.current) setIsValidating(false);
      }
    };

    if (debounceTimer.current) clearTimeout(debounceTimer.current);
    debounceTimer.current = setTimeout(run, 300);

    return () => {
      if (debounceTimer.current) clearTimeout(debounceTimer.current);
    };
  }, [layoutString, layoutName, weights, activeJobId, selectedKeyboard, isDatasetLoaded]);

  return (
    <AnalysisContext.Provider
      value={{
        activeResult,
        referenceResult,
        heatmap: activeResult?.heatmap,
        isValidating,
      }}
    >
      {children}
    </AnalysisContext.Provider>
  );
}

export const useAnalysis = () => {
  const ctx = useContext(AnalysisContext);
  if (!ctx) throw new Error("useAnalysis must be used within AnalysisProvider");
  return ctx;
};
