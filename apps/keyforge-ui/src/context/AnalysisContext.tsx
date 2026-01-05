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
  const { weights, availableLayouts, selectedKeyboard } = useLibrary();
  const { hiveUrl } = useSystem();
  const backend = useBackend();
  const { addToast } = useToast();

  const [activeResult, setActiveResult] = useState<ValidationResult | null>(
    null,
  );
  const [referenceResult, setReferenceResult] =
    useState<ValidationResult | null>(null);
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

  // Load Reference (Prefer Colemak-DH, then Qwerty, then Default)
  useEffect(() => {
    if (!isDatasetLoaded) return; // Prevent "No runtime loaded" race condition

    const runRef = async () => {
      let targetName = "Colemak-DH";
      let targetLayout = availableLayouts["Colemak-DH"];

      if (!targetLayout) {
        targetName = "Qwerty";
        targetLayout = availableLayouts["Qwerty"];
      }

      if (!targetLayout) {
        targetName = "default";
        targetLayout = availableLayouts["default"];
      }
      
      if (!targetLayout && Object.keys(availableLayouts).length > 0) {
         targetName = Object.keys(availableLayouts)[0];
         targetLayout = availableLayouts[targetName];
      }

      if (targetLayout) {
        try {
          const ref = await backend.validateLayout(
            targetLayout,
            undefined,
            hiveUrl,
            selectedKeyboard,
          );
          setReferenceResult({ ...ref, layout_name: targetName });
        } catch (e) {
          console.warn(`Failed to load reference '${targetName}':`, JSON.stringify(e));
        }
      }
    };
    runRef();
  }, [availableLayouts, selectedKeyboard, isDatasetLoaded]);

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
