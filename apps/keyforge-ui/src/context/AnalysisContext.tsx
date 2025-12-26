import {
  createContext,
  useContext,
  useState,
  useEffect,
  ReactNode,
  useRef,
} from "react";
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
  const { layoutString, layoutName, activeJobId } = useSession();
  const { weights, availableLayouts, selectedKeyboard } = useLibrary();
  const { hiveUrl } = useSystem();
  const backend = useBackend();

  const [activeResult, setActiveResult] = useState<ValidationResult | null>(
    null,
  );
  const [referenceResult, setReferenceResult] =
    useState<ValidationResult | null>(null);
  const [isValidating, setIsValidating] = useState(false);

  const validationReqId = useRef(0);
  const debounceTimer = useRef<NodeJS.Timeout | null>(null);

  // Run Validation when layout changes
  useEffect(() => {
    if (!layoutString) {
      setActiveResult(null);
      return;
    }
    if (activeJobId) return; // Don't validate while job is running (job updates handle this)

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
        console.error("Validation error:", e);
      } finally {
        if (currentId === validationReqId.current) setIsValidating(false);
      }
    };

    if (debounceTimer.current) clearTimeout(debounceTimer.current);
    debounceTimer.current = setTimeout(run, 300);

    return () => {
      if (debounceTimer.current) clearTimeout(debounceTimer.current);
    };
  }, [layoutString, layoutName, weights, activeJobId, selectedKeyboard]);

  // Load Reference (Qwerty)
  useEffect(() => {
    const runRef = async () => {
      if (availableLayouts["Qwerty"]) {
        try {
          const ref = await backend.validateLayout(
            availableLayouts["Qwerty"],
            undefined,
            hiveUrl,
            selectedKeyboard,
          );
          setReferenceResult(ref);
        } catch (e) {
          console.warn("Failed to load reference:", e);
        }
      }
    };
    runRef();
  }, [availableLayouts, selectedKeyboard]);

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
