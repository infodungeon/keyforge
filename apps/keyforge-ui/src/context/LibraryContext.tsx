import {
  createContext,
  useContext,
  useState,
  useEffect,
  ReactNode,
  useCallback,
  useRef,
} from "react";
import { ScoringWeights, SearchParams, KeyboardGeometry } from "../types";
import { keycodeService } from "../utils";
import { useToast } from "./ToastContext";
import { useBackend } from "./BackendContext";
import { useSystem } from "./SystemContext";

interface LibraryContextType {
  weights: ScoringWeights | null;
  searchParams: SearchParams | null;
  setWeights: (w: ScoringWeights) => void;
  setSearchParams: (p: SearchParams) => void;

  keyboards: string[];
  selectedKeyboard: string;
  selectKeyboard: (name: string) => void;
  keyboardGeometry: KeyboardGeometry | null;

  corpora: string[];
  selectedCorpus: string;
  selectCorpus: (filename: string) => void;

  costMatrices: string[];
  selectedCostMatrix: string;
  selectCostMatrix: (filename: string) => void;

  availableExtras: string[];
  selectedExtras: string[];
  toggleExtra: (name: string) => void;

  availableLayouts: Record<string, string | undefined>;
  standardLayouts: string[];

  refreshLibrary: () => Promise<number>;

  saveUserLayout: (name: string, layout: string) => Promise<void>;
  deleteUserLayout: (name: string) => Promise<void>;

  libraryVersion: number;

  hiveSecret: string;
  setHiveSecret: (s: string) => void;

  hiveUrl: string;
  isBootstrapping: boolean;
  bootstrapError: string | null;
  retryBootstrap: () => Promise<void>;
}

const LibraryContext = createContext<LibraryContextType | undefined>(undefined);

export function LibraryProvider({ children }: { children: ReactNode }) {
  const { addToast } = useToast();
  const backend = useBackend();
  const { hiveUrl, hiveSecret, setHiveSecret } = useSystem();

  // Config State
  const [weights, setWeights] = useState<ScoringWeights | null>(null);
  const [searchParams, setSearchParams] = useState<SearchParams | null>(null);

  // Selection State
  const [keyboards, setKeyboards] = useState<string[]>([]);
  const [selectedKeyboard, setSelectedKeyboard] = useState(
    () => localStorage.getItem("last_keyboard") || "ortho_30",
  );

  const [corpora, setCorpora] = useState<string[]>([]);
  const [selectedCorpus, setSelectedCorpus] = useState(
    () => localStorage.getItem("last_corpus") || "text/en_std",
  );

  const [costMatrices, setCostMatrices] = useState<string[]>([]);
  const [selectedCostMatrix, setSelectedCostMatrix] = useState(
    () => localStorage.getItem("last_cost") || "cost_matrix.json",
  );

  const [availableExtras, setAvailableExtras] = useState<string[]>([]);
  const [selectedExtras, setSelectedExtras] = useState<string[]>(() =>
    JSON.parse(localStorage.getItem("selected_extras") || "[]"),
  );

  // Layout Data
  const [availableLayouts, setAvailableLayouts] = useState<
    Record<string, string | undefined>
  >({});
  const [standardLayouts, setStandardLayouts] = useState<string[]>([]);

  const [libraryVersion, setLibraryVersion] = useState(0);
  const [keyboardGeometry, setKeyboardGeometry] = useState<KeyboardGeometry | null>(null);

  // Bootstrap State
  const [isBootstrapping, setIsBootstrapping] = useState(false);
  const [bootstrapError, setBootstrapError] = useState<string | null>(null);
  const [isInitialized, setIsInitialized] = useState(false);

  const refreshLibrary = useCallback(async () => {
    try {
      const [kbs, corps, costs, extras] = await Promise.all([
        backend.listKeyboards(hiveUrl),
        backend.listCorpora(hiveUrl),
        backend.listCostMatrices(hiveUrl),
        backend.listKeymapExtras(hiveUrl),
      ]);

      // SYSTEMIC VALIDATION
      if (kbs.length === 0) throw new Error("No keyboards found in library.");
      if (corps.length === 0) throw new Error("No corpora found in library.");
      if (costs.length === 0) throw new Error("No cost matrices found in library.");

      setKeyboards(kbs);
      setCorpora(corps);
      setCostMatrices(costs);
      setAvailableExtras(extras);

      setLibraryVersion((v) => v + 1);
      return kbs.length;
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      console.error("Library Refresh Error:", e);
      addToast("error", `Library Error: ${msg}`);
      return 0;
    }
  }, [backend, hiveUrl, addToast]);

  const performBootstrap = useCallback(async () => {
    setIsBootstrapping(true);
    setBootstrapError(null);
    try {
      await backend.bootstrapAssets(hiveUrl);
      const count = await refreshLibrary();
      if (count > 0) {
        addToast("success", "Workspace initialized successfully.");
      } else {
        throw new Error("Bootstrap completed but library is empty.");
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setBootstrapError(`Bootstrap Failed: ${msg}`);
      addToast("error", `Bootstrap Failed: ${msg}`);
    } finally {
      setIsBootstrapping(false);
    }
  }, [hiveUrl, refreshLibrary, addToast, backend]);

  const initStarted = useRef(false);

  useEffect(() => {
    if (initStarted.current) return;
    initStarted.current = true;

    const init = async () => {
      try {
        await new Promise((r) => setTimeout(r, 100));

        const conf = await backend.getDefaultConfig(hiveUrl);
        setWeights(conf.weights);
        setSearchParams(conf.search);

        const reg = await backend.getKeycodes(hiveUrl);
        if (!reg.definitions || reg.definitions.length === 0) {
            throw new Error("Keycode Registry is empty.");
        }
        keycodeService.loadDefinitions(reg.definitions);

        const kbCount = await refreshLibrary();

        let isValid = kbCount > 0;
        if (isValid) {
          try {
            const kbs = await backend.listKeyboards(hiveUrl);
            const target = kbs.includes(selectedKeyboard)
              ? selectedKeyboard
              : kbs[0];
            await backend.getKeyboardGeometry(target, hiveUrl);
          } catch (e) {
            isValid = false;
          }
        }

        if (!isValid) {
          await performBootstrap();
        }
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        console.error("Library Init Error:", e);
        addToast("error", `Initialization Failed: ${msg}`);
      } finally {
        setIsInitialized(true);
      }
    };
    init();
  }, [backend, hiveUrl, refreshLibrary, performBootstrap, selectedKeyboard, addToast]);

  useEffect(() => {
    if (!selectedKeyboard) return;
    const loadKb = async () => {
      try {
        const [all, geo] = await Promise.all([
          backend.getAllLayoutsScoped(selectedKeyboard, hiveUrl),
          backend.getKeyboardGeometry(selectedKeyboard, hiveUrl),
        ]);
        setAvailableLayouts(all);
        setKeyboardGeometry(geo);
        setStandardLayouts(Object.keys(all).filter((k) => k !== "Custom"));
      } catch (e) {
        console.error("Keyboard Load Error:", e);
        addToast("error", `Failed to load layouts for ${selectedKeyboard}`);
      }
    };
    loadKb();
  }, [selectedKeyboard, libraryVersion, backend, hiveUrl, addToast]);

  const selectKeyboard = (name: string) => {
    setSelectedKeyboard(name);
    localStorage.setItem("last_keyboard", name);
    setLibraryVersion((v) => v + 1);
  };

  const selectCorpus = (filename: string) => {
    setSelectedCorpus(filename);
    localStorage.setItem("last_corpus", filename);
    setLibraryVersion((v) => v + 1);
  };

  const selectCostMatrix = (filename: string) => {
    setSelectedCostMatrix(filename);
    localStorage.setItem("last_cost", filename);
    setLibraryVersion((v) => v + 1);
  };

  const toggleExtra = (name: string) => {
    const next = selectedExtras.includes(name)
      ? selectedExtras.filter((e) => e !== name)
      : [...selectedExtras, name];

    setSelectedExtras(next);
    localStorage.setItem("selected_extras", JSON.stringify(next));
    setLibraryVersion((v) => v + 1);
  };

  const saveUserLayout = async (name: string, layout: string) => {
    try {
      await backend.saveUserLayout(selectedKeyboard, name, layout, hiveUrl);
      addToast("success", `Layout '${name}' saved.`);
      const all = await backend.getAllLayoutsScoped(selectedKeyboard, hiveUrl);
      setAvailableLayouts(all);
    } catch (e) {
      addToast("error", `Save failed: ${e}`);
    }
  };

  const deleteUserLayout = async (name: string) => {
    try {
      await backend.deleteUserLayout(selectedKeyboard, name, hiveUrl);
      addToast("info", `Layout '${name}' deleted.`);
      const all = await backend.getAllLayoutsScoped(selectedKeyboard, hiveUrl);
      setAvailableLayouts(all);
    } catch (e) {
      addToast("error", `Delete failed: ${e}`);
    }
  };

  return (
    <LibraryContext.Provider
      value={{
        weights,
        searchParams,
        setWeights,
        setSearchParams,
        keyboards,
        selectedKeyboard,
        selectKeyboard,
        keyboardGeometry,
        corpora,
        selectedCorpus,
        selectCorpus,
        costMatrices,
        selectedCostMatrix,
        selectCostMatrix,
        availableExtras,
        selectedExtras,
        toggleExtra,
        availableLayouts,
        standardLayouts,
        refreshLibrary,
        saveUserLayout,
        deleteUserLayout,
        libraryVersion,
        hiveSecret,
        setHiveSecret,
        hiveUrl,
        isBootstrapping,
        bootstrapError,
        retryBootstrap: performBootstrap,
      }}
    >
      {isInitialized ? (
        children
      ) : (
        <div className="h-screen bg-[#020617] text-slate-500 flex items-center justify-center text-xs font-mono">
          INITIALIZING WORKSPACE...
        </div>
      )}
    </LibraryContext.Provider>
  );
}

export const useLibrary = () => {
  const ctx = useContext(LibraryContext);
  if (!ctx) throw new Error("useLibrary must be used within LibraryProvider");
  return ctx;
};