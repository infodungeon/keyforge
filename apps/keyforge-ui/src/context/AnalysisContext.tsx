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

import { SpaceHandPreference } from "../services/stats";

interface AnalysisContextType {
  activeResult: ValidationResult | null;
  referenceResult: ValidationResult | null;
  heatmap: number[] | undefined;
  isValidating: boolean;

  includeThumbs: boolean;
  setIncludeThumbs: (b: boolean) => void;
  spaceHand: SpaceHandPreference;
  setSpaceHand: (p: SpaceHandPreference) => void;
}

const AnalysisContext = createContext<AnalysisContextType | undefined>(
  undefined,
);

export function AnalysisProvider({ children }: { children: ReactNode }) {
  const { layoutString, layoutName, activeJobId, isDatasetLoaded } =
    useSession();
  const { weights, selectedKeyboard, keyboardGeometry } = useLibrary();
  const { hiveUrl } = useSystem();
  const backend = useBackend();
  const { addToast } = useToast();

  const [activeResult, setActiveResult] = useState<ValidationResult | null>(
    null,
  );
  // Reference result is now null by default as per user request to remove auto-calculation
  const [referenceResult] = useState<ValidationResult | null>(null);
  const [isValidating, setIsValidating] = useState(false);

  const [includeThumbs, setIncludeThumbs] = useState(true);
  const [spaceHand, setSpaceHand] = useState<SpaceHandPreference>("bilateral");

  const validationReqId = useRef(0);
  const debounceTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Run Validation when layout changes or physics settings change
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
        const tokens = layoutString.trim().split(/\s+/);

        // --- Dynamic Masking Logic ---
        console.log(
          `[Analysis] Running validation. IncludeThumbs=${includeThumbs} SpaceHand=${spaceHand}`,
        );
        const maskedTokens = tokens.map((token, idx) => {
          const keyDef = keyboardGeometry?.keys[idx];
          if (!keyDef) return token;

          // 1. Filter Thumbs (Finger 0)
          if (!includeThumbs) {
            if (keyDef.finger === 0) return "KC_NO";
          }

          // 2. Filter Space Hand Preference
          const isSpace = ["KC_SPC", "SPACE", "SPC", "KC_SPACE"].includes(
            token.toUpperCase(),
          );
          if (isSpace && spaceHand !== "bilateral") {
            const targetHand = spaceHand === "left" ? 0 : 1;
            if (keyDef.hand !== targetHand) {
              return "KC_NO";
            }
          }

          return token;
        });

        const qmkStr = fromDisplayString(maskedTokens.join(" "));
        console.log(
          "[Analysis] Masked tokens count:",
          maskedTokens.filter((t) => t === "KC_NO").length,
        );
        console.log("[Analysis] Validating Layout (QMK):", qmkStr);

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
        addToast(
          "error",
          `Analysis Failed: ${e instanceof Error ? e.message : String(e)}`,
        );
      } finally {
        if (currentId === validationReqId.current) setIsValidating(false);
      }
    };

    if (debounceTimer.current) clearTimeout(debounceTimer.current);
    debounceTimer.current = setTimeout(run, 300);

    return () => {
      if (debounceTimer.current) clearTimeout(debounceTimer.current);
    };
  }, [
    layoutString,
    layoutName,
    weights,
    activeJobId,
    selectedKeyboard,
    isDatasetLoaded,
    includeThumbs,
    spaceHand,
    keyboardGeometry,
  ]);

  return (
    <AnalysisContext.Provider
      value={{
        activeResult,
        referenceResult,
        heatmap: activeResult?.heatmap,
        isValidating,
        includeThumbs,
        setIncludeThumbs,
        spaceHand,
        setSpaceHand,
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
